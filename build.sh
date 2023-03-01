docker build -t maxday/pizza .
dockerId=$(docker create maxday/pizza)
docker cp $dockerId:/bootstrap.zip .