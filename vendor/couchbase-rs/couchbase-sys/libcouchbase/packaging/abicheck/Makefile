OLDVER ?= $(shell git tag --list --sort=-v:refname | head -1)
NEWVER ?= $(shell git describe)
NEWROOT ?= $(shell realpath ../../../libcouchbase-${NEWVER})
OLDROOT ?= $(shell realpath ../../../libcouchbase-${OLDVER})

all: report

templates:
	PREFIX=${NEWROOT} VERSION=${NEWVER} erb -P -T 2 template.xml.erb > ${NEWVER}.xml
	PREFIX=${OLDROOT} VERSION=${OLDVER} erb -P -T 2 template.xml.erb > ${OLDVER}.xml

report:
	abi-compliance-checker -lib libcouchbase -old ${OLDVER}.xml -new ${NEWVER}.xml
