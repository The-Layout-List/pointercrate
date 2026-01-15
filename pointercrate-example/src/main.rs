
use pointercrate_core::pool::PointercratePool;
use pointercrate_core::error::CoreError;
use pointercrate_core_api::{error::ErrorResponder, maintenance::MaintenanceFairing, preferences::PreferenceManager};
use pointercrate_demonlist_api::GeolocationProvider;
use rocket::{async_trait, serde, Request};
use std::net::IpAddr;
use dotenv;

/// A catcher for 404 errors (e.g. when a user tried to navigate to a URL that
/// does not exist)
///
/// An [`ErrorResponder`] will return either a JSON or an HTML error page,
/// depending on what `Accept` headers are set on the request.
#[rocket::catch(404)]
async fn catch_404() -> ErrorResponder {
    // `CoreError` contains various generic error conditions that might happen
    // anywhere across the website. `CoreError::NotFound` is a generic 404 NOT FOUND
    // error with code 40400.
    CoreError::NotFound.into()
}

/// Failures in json deserialization of request bodies will just return
/// an immediate 422 response. This catcher is needed to translate them into a pointercrate
/// error response.
#[rocket::catch(422)]
async fn catch_422() -> ErrorResponder {
    CoreError::UnprocessableEntity.into()
}

/// Failures from the authorization FromRequest implementations can return 401s
#[rocket::catch(401)]
async fn catch_401() -> ErrorResponder {
    CoreError::Unauthorized.into()
}

/// A very simplistic geolocation provider based on https://ipwho.is/
///
/// Note that ipwho.is is only free for testing, non-commercial use-cases, and
/// up to 1000 requests / mo. In a production environment, it would be up to you to
/// implement appropriate rate limits / use a service that matches your usecase!
///
/// Note that when running this locally, all requests will come from 127.0.0.1, which
/// obviously cannot be geolocated.
struct IpWhoIsGeolocationProvider;

#[async_trait]
impl GeolocationProvider for IpWhoIsGeolocationProvider {
    async fn geolocate(&self, req: &Request<'_>) -> Option<(String, Option<String>)> {
        #[derive(serde::Deserialize)]
        struct IpWhoIsResponse {
            country_code: String,
            region_code: Option<String>,
        }

        let remote_ip: IpAddr = req.guard().await.succeeded()?;

        let resp = reqwest::get(format!("https://ipwho.is/{}", remote_ip)).await.ok()?;

        let data: IpWhoIsResponse = resp.json().await.ok()?;

        Some((data.country_code, data.region_code))
    }
}

#[rocket::launch]
async fn rocket() -> _ {
    // Load the configuration from your .env file
    dotenv::dotenv().unwrap();

    // Initialize a database connection pool to the database specified by the
    // DATABASE_URL environment variable
    let pool = PointercratePool::init().await;

    // Set up the HTTP server
    let rocket = rocket::build()
        // Tell it about the connection pool to use (individual handlers can get hold of this pool by declaring an argument of type `&State<PointercratePool>`)
        .manage(pool)
        // Register our 404 catcher
        .register("/", rocket::catchers![catch_401, catch_404, catch_422]);

    // Define the permissions in use on our website. We just use the default setup
    // from `pointercrate_user` and `pointercrate_demonlist`, but if you for example
    // do not want list administrators to be able to promote helpers to moderators
    // in autonomy, you could use a custom [`PermissionsManager`] where the
    // `LIST_ADMINISTRATOR` permission does not assign `LIST_MODERATOR` and
    // `LIST_HELPER`. For more information on pointercrate' permissions system, see
    // the documentation of the [`PermissionsManager`] structure.
    let mut permissions_manager = pointercrate_user::default_permissions_manager();
    permissions_manager.merge_with(pointercrate_demonlist::default_permissions_manager());

    let rocket = rocket.manage(permissions_manager);

    // Define the preferences our website supports. Preferences are sent to us from
    // the client via cookies.
    let preference_manager = PreferenceManager::default();

    let rocket = rocket.manage(preference_manager);

    // Register the geolocation provider, so that we can geolocate player claims. The type erasure is important, otherwise you'll get internal server errors!
    let rocket = rocket.manage(Box::new(IpWhoIsGeolocationProvider) as Box<dyn GeolocationProvider>);

    // Changing `false` to `true` here will put your website into "maintenance mode", which will disable all mutating request handlers and always return 503 SERVICE UNAVAILABLE responses for non-GET requests.
    let rocket = rocket.attach(MaintenanceFairing::new(false));

    // Register all the endpoints related to the demonlist to our server (this is
    // optional, but without registering the demonlist related endpoint your website
    // will just be User Account Simulator 2024).
    let rocket = pointercrate_demonlist_api::setup(rocket);

    // Register all the endpoints related to the user account system to our server
    pointercrate_user_api::setup(rocket)

}