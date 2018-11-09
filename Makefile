SHELL := /bin/bash

.PHONY : 
doc : build_doc sync_doc

sync_doc:
	./sync.sh

build_doc:
	mdbook build ./docs

