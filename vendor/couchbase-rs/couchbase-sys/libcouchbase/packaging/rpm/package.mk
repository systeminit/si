.PHONY: dist-rpm

# Defined by CMake. For autotools, these will just be the normal build tree
BUILDROOT		?= $(shell pwd)
SRCROOT			?= $(shell pwd)

# Variables derived
GITPARSE		:= $(SRCROOT)/packaging/parse-git-describe.pl
RPM_WORKSPACE	:= $(BUILDROOT)/build-rpm
RPM_DIR			:= $(RPM_WORKSPACE)/
RPM_VER			:= $(shell $(GITPARSE) --rpm-ver --input $(REVDESCRIBE))
RPM_REL			:= $(shell $(GITPARSE) --rpm-rel --input $(REVDESCRIBE))
TAR_VERSION		:= $(shell $(GITPARSE) --tar --input $(REVDESCRIBE))

EXTRA_RPMDEFS	:=

dist-rpm: dist
	rm -rf $(RPM_WORKSPACE)
	mkdir -p $(RPM_DIR)
	mkdir $(RPM_DIR)/SOURCES
	mkdir $(RPM_DIR)/BUILD
	mkdir $(RPM_DIR)/RPMS
	mkdir $(RPM_DIR)/SRPMS
	cp $(BUILDROOT)/$(PACKAGE)-$(TAR_VERSION).tar.gz $(RPM_DIR)/SOURCES
	sed \
		's/@VERSION@/$(RPM_VER)/g;s/@RELEASE@/$(RPM_REL)/g;s/@TARREDAS@/libcouchbase-$(TAR_VERSION)/g' \
		< packaging/rpm/$(PACKAGE).spec.in > $(RPM_WORKSPACE)/$(PACKAGE).spec

	(cd $(RPM_WORKSPACE) && \
		rpmbuild ${RPM_FLAGS} -ba \
		--define "_topdir $(RPM_DIR)" \
		--define "_source_filedigest_algorithm md5" \
		--define "_binary_filedigest_algorithm md5" \
		$(EXTRA_RPMDEFS) \
		$(PACKAGE).spec \
	)

	mv $(RPM_DIR)/RPMS/*/*.rpm $(BUILDROOT)
	mv $(RPM_DIR)/SRPMS/*.rpm $(BUILDROOT)
	rm -rf $(RPM_WORKSPACE)
