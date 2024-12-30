build: 
	@odin build src -out:atlas

run: build
	@./atlas
	
