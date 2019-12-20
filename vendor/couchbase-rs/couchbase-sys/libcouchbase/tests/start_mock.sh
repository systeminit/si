#! /bin/sh
#
#     Copyright 2011 Couchbase, Inc.
#
#   Licensed under the Apache License, Version 2.0 (the "License");
#   you may not use this file except in compliance with the License.
#   You may obtain a copy of the License at
#
#       http://www.apache.org/licenses/LICENSE-2.0
#
#   Unless required by applicable law or agreed to in writing, software
#   distributed under the License is distributed on an "AS IS" BASIS,
#   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#   See the License for the specific language governing permissions and
#   limitations under the License.
#

# We don't want to run memory debugging on java ;)
unset LD_PRELOAD
unset MALLOC_DEBUG
unset UMEM_DEBUG

# This is a wrapper script to start the Couchbase Mock server.
# We could have started it directly from the C code, but by using
# a script it's a bit easier to test it manually ;)
if [ -z "$srcdir" ]; then
    srcdir="."
fi

for p in "$srcdir/tests" "$srcdir" "tests" "."; do
    if [ -f "$p/CouchbaseMock.jar" ]; then
        COUCHBASEMOCK="$p/CouchbaseMock.jar"
    fi
done

exec java \
       -client \
       -jar "$COUCHBASEMOCK" \
        --nodes=4 \
        --host=localhost \
        --port=0 \
        "$@"
