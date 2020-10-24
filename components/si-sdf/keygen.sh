#!/bin/bash

OUTPATH=$1

openssl genrsa -out $OUTPATH/private.pem 2048
openssl rsa -in $OUTPATH/private.pem -outform PEM -pubout -out $OUTPATH/public.pem
