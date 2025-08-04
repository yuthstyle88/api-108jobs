-- Add new billing status values
ALTER TYPE billing_status ADD VALUE 'RequestChange' AFTER 'RevisionRequested';
ALTER TYPE billing_status ADD VALUE 'Updated' AFTER 'RequestChange';