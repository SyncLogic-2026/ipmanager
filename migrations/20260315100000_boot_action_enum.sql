DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'boot_action') THEN
        CREATE TYPE boot_action AS ENUM ('LOCAL', 'INSTALL', 'SHELL');
    END IF;
END $$;

ALTER TABLE hosts
    ADD COLUMN IF NOT EXISTS next_boot_action boot_action;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'hosts'
          AND column_name = 'next_boot_action'
          AND udt_name <> 'boot_action'
    ) THEN
        ALTER TABLE hosts
            ALTER COLUMN next_boot_action DROP DEFAULT;
        ALTER TABLE hosts
            ALTER COLUMN next_boot_action TYPE boot_action
            USING CASE
                WHEN next_boot_action IS NULL THEN NULL
                ELSE UPPER(next_boot_action)::boot_action
            END;
    END IF;
END $$;

ALTER TABLE hosts
    ALTER COLUMN next_boot_action SET DEFAULT 'LOCAL';
