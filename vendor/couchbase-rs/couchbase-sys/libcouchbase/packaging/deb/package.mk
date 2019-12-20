.PHONY: dist-deb

# Defined by CMake. For autotools, these will just be the normal build tree
BUILDROOT		?= $(shell pwd)
SRCROOT			?= $(shell pwd)

# Variables derived
GITPARSE		:= $(SRCROOT)/packaging/parse-git-describe.pl
DEB_WORKSPACE	:= $(BUILDROOT)/build-deb
DEB_DIR			:= $(DEB_WORKSPACE)/$(PACKAGE)-$(VERSION)
DEB_VERSION		:= $(shell $(GITPARSE) --deb --input $(REVDESCRIBE))
TAR_VERSION		:= $(shell $(GITPARSE) --tar --input $(REVDESCRIBE))

dist-deb: dist
	cd $(BUILDROOT)
	rm -rf $(DEB_WORKSPACE)
	mkdir -p $(DEB_DIR)
	cp -r $(SRCROOT)/packaging/deb $(DEB_DIR)/debian
	cp $(BUILDROOT)/$(PACKAGE)-$(TAR_VERSION).tar.gz $(DEB_WORKSPACE)/$(PACKAGE)_$(DEB_VERSION).orig.tar.gz
	(cd $(DEB_WORKSPACE); tar zxvf $(PACKAGE)_$(DEB_VERSION).orig.tar.gz)
	(\
		cd $(DEB_DIR); \
		dch \
		--package=libcouchbase \
		--create \
		--newversion="$(DEB_VERSION)" \
		"Release package for $(DEB_VERSION)" && \
		dpkg-buildpackage -rfakeroot ${DEB_FLAGS}\
	)
	mv $(DEB_WORKSPACE)/*.{changes,deb,dsc,tar.gz} `pwd`
	rm -rf $(DEB_WORKSPACE)
