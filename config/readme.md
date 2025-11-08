# Update pages
cd syncSlide/
cargo build

# Update caddy
sudo cp syncSlide/config/syncSlide.conf /etc/caddy/conf.d
sudo chown root:root /etc/caddy/conf.d/syncSlide.conf
sudo systemctl reload caddy
