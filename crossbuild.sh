#!/bin/bash

cross build -v -r --target=x86_64-pc-windows-gnu
cross build -v -r --target=x86_64-unknown-linux-gnu
