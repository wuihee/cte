#!/bin/bash

mkdir data
rm -f data/app.db
touch data/app.db
sqlx migrate run
