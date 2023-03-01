docker buid --platform linux/amd64 -t maxday/pizza .
dockerId=$(docker create --platform linux/amd64 maxday/pizza)
docker cp $dockerId:/bootstrap.zip .