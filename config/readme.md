# Update pages
sudo cp -r syncSlide/src/* /usr/share/caddy/public/syncslide/

# Update caddy
sudo chown root:root /etc/caddy/conf.d/syncslide/
sudo systemctl reload caddy