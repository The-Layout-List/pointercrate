-- Add up migration script here
ALTER TABLE records ADD COLUMN enjoyment SMALLINT CHECK (enjoyment BETWEEN 0 AND 10);
ALTER TABLE record_modifications ADD COLUMN enjoyment SMALLINT CHECK (enjoyment BETWEEN 0 AND 10);

CREATE OR REPLACE FUNCTION audit_record_modification() RETURNS trigger AS $record_modification_trigger$
    DECLARE
        progress_change SMALLINT;
        video_change VARCHAR(200);
        status_change RECORD_STATUS;
        player_change INT;
        demon_change INTEGER;
        enjoyment_change SMALLINT;
    BEGIN
        if (OLD.progress <> NEW.progress) THEN
            progress_change = OLD.progress;
        END IF;

        IF (OLD.video <> NEW.video) THEN
            video_change = OLD.video;
        END IF;

        IF (OLD.status_ <> NEW.status_) THEN
            status_change = OLD.status_;
        END IF;

        IF (OLD.player <> NEW.player) THEN
            player_change = OLD.player;
        END IF;

        IF (OLD.demon <> NEW.demon) THEN
            demon_change = OLD.demon;
        END IF;

        IF (OLD.enjoyment <> NEW.enjoyment) THEN
            enjoyment_change = OLD.enjoyment;
        END IF;

        INSERT INTO record_modifications (userid, id, progress, video, status_, player, enjoyment, demon)
            (SELECT id, NEW.id, progress_change, video_change, status_change, player_change, enjoyment_change, demon_change
            FROM active_user LIMIT 1);

        RETURN NEW;
    END;
$record_modification_trigger$ LANGUAGE plpgsql;