curl -X DELETE -H "Content-Type: application/json" -H "Authorization: Bearer $DO_TOKEN" "https://api.digitalocean.com/v2/droplets?tag_name=podcastrs-podrocket"
docker-machine create --driver=digitalocean --digitalocean-access-token=$DO_TOKEN --digitalocean-size=s-1vcpu-1gb --digitalocean-tags podcastrs-podrocket podrocket
eval "$(docker-machine env podrocket)"
cd docker/podrocket
docker-compose up --build -d
