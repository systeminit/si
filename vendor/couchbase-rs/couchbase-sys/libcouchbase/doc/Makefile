SRCROOT=$(shell pwd)
VERSINFO=$(shell $(SRCROOT)/packaging/parse-git-describe.pl --tar)
ifneq ($(strip $(VERSINFO)),)
  VERLINE=PROJECT_NUMBER=$(VERSINFO)
endif

DOXYGEN?=doxygen
DOXYFILE=doc/Doxyfile
OUTDIR_PUB = doc/public
OUTDIR_PRIV = doc/internal
INTERNAL_SRC = \
			   src/rdb \
			   src/bootstrap.h \
			   src/lcbio \
			   src/netbuf \
			   src/mc \
			   src/retryq.h \
			   src/bucketconfig/clconfig.h \
			   src/mcserver \
			   src/list.h \
			   src/sllist.h \
			   src/sllist-inl.h \
			   src/hostlist.h \
			   include/memcached/protocol_binary.h

all: public internal
public:
	(cat $(DOXYFILE);\
		echo $(VERLINE);\
		echo OUTPUT_DIRECTORY=$(OUTDIR_PUB)) | $(DOXYGEN) -
	cp doc/style.css $(OUTDIR_PUB)/html/style.css



internal:
	(cat $(DOXYFILE);\
		echo $(VERLINE); \
		echo OUTPUT_DIRECTORY=$(OUTDIR_PRIV); \
		echo INTERNAL_DOCS=yes; \
		echo INPUT += $(INTERNAL_SRC);) | $(DOXYGEN) -
	cp doc/style.css $(OUTDIR_PRIV)/html/style.css

clean:
	rm -r -f $(OUTDIR_PRIV)/html $(OUTDIR_PUB)/html
