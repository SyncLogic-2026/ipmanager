alter table subnets
  add column if not exists dns_primary inet;
