# Deployment

## Set up a google compute engine VM with Debian 12

- 300usd in free credits
- ec2-micro, us-central1, 30gb is free? https://cloud.google.com/free/docs/free-cloud-features#compute


## Test ssh connection

```shell
gcloud compute instances list
NAME        ZONE           MACHINE_TYPE  PREEMPTIBLE  INTERNAL_IP  EXTERNAL_IP     STATUS
gcloud compute ssh <username>@<NAME>
gcloud compute scp test.txt <username>@<NAME>:~/
```


## Install docker on debian 12 instance

```shell
sudo apt update
sudo apt upgrade -y

sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian bookworm stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

sudo apt update
sudo apt-get install -y docker-ce
sudo usermod -aG docker $USER
# logout and back in to assume docker group membership
```

## Deploy the app

```shell
# build the image
docker build -t qposit .
docker save -o qposit_app.tar qposit:latest
gcloud compute scp qposit_app.tar Caddyfile docker-compose.yml implygate@basic-site:~/
gcloud compute scp db/db.db implygate@basic-site:~/db/db.db
gcloud compute scp .env implygate@basic-site:~/.env

# ssh into VM
gcloud compute ssh implygate@basic-site
docker load -i qposit_app.tar
docker compose up -d

# take it down
docker compose down
```
