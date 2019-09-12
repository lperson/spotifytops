./docker_rm.sh

docker run -it \
	-p 8080:8888  \
	--name spotifytops 	\
 	-v /Users/larryperson/spotifytopsconfig.toml:/root/spotifytopsconfig.toml \
	-e SPOTIFY_TOPS_TEMPLATE_DIR='/root/templates' 	\
	-e SPOTIFY_TOPS_LISTEN_ADDR='0.0.0.0' 	\
	-e PORT='8888' 	\
	spotifytops
