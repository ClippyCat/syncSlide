# Update pages
sudo cp -r syncSlide/src/* /usr/share/caddy/public/syncslide/

# Update caddy
sudo chown root:root /etc/caddy/conf.d/syncSlide.conf
sudo systemctl reload caddy