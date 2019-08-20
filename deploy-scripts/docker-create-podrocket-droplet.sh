docker-machine create --driver=digitalocean --digitalocean-access-token=$DO_TOKEN --digitalocean-size=s-1vcpu-1gb podrocket
eval $(docker-machine env podrocket)
