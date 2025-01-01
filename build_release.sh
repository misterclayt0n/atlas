#!/usr/bin/env bash

odin build src/main_release -out:atlas_release.bin -strict-style -vet -no-bounds-check -o:speed
