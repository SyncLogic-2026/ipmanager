insert into users (username, password_hash, role, is_active)
values ('admin', crypt('admin123', gen_salt('bf')), 'admin', true)
on conflict (username) do nothing;

insert into subnets (name, cidr, gateway, dns_primary)
values ('Default LAN', '192.168.0.0/24', '192.168.0.1', '1.1.1.1')
on conflict (name) do nothing;
