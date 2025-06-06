/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

package com.facebook.buck.util.zip;

import com.facebook.buck.util.function.ThrowingSupplier;
import java.io.IOException;
import java.io.InputStream;

/**
 * Encapsulates a file or directory to be added as a single entry to a jar by {@link JarBuilder}.
 */
public class JarEntrySupplier {
  private final CustomZipEntry entry;
  private final ThrowingSupplier<InputStream, IOException> inputStreamSupplier;
  private final boolean readOnly;

  public JarEntrySupplier(
      CustomZipEntry entry, ThrowingSupplier<InputStream, IOException> inputStreamSupplier) {
    this(entry, false, inputStreamSupplier);
  }

  public JarEntrySupplier(
      CustomZipEntry entry,
      boolean readOnly,
      ThrowingSupplier<InputStream, IOException> inputStreamSupplier) {
    this.entry = entry;
    this.readOnly = readOnly;
    this.inputStreamSupplier = inputStreamSupplier;
  }

  public CustomZipEntry getEntry() {
    return entry;
  }

  public ThrowingSupplier<InputStream, IOException> getInputStreamSupplier() {
    return inputStreamSupplier;
  }

  public boolean isReadOnly() {
    return readOnly;
  }
}
