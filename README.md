# Build image

docker build -t muntakkibot .

# Run container with volumes

docker run -e TELOXIDE_TOKEN="<your_actual_token>" -v /<absolute_path_to_image_folder>:/usr/src/muntakkibot/images mun_takki_bot
