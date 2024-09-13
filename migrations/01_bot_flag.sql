-- Add bot column to determing if someone is a bot
ALTER TABLE `user_visits` ADD COLUMN `is_bot` INTEGER NOT NULL DEFAULT TRUE;
