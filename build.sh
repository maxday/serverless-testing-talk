docker buildx build --platform linux/amd64 -t maxday/pizza . --load
dockerId=$(docker create --platform linux/amd64 maxday/pizza)
docker cp $dockerId:/bootstrap.zip .