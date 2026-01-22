alter table hosts
  add column if not exists boot_target text not null default 'local';
