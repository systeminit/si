/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2016-2019 Couchbase, Inc.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *       http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

#include "contrib/lcb-jsoncpp/lcb-jsoncpp.h"
#include "placeholders.h"
#include <algorithm>
#include <stdexcept>

namespace Pillowfight
{

#define JSON_VALUE_SIZE 16

/**
 * Per-thread mutable state to generate the document. The populateIov()
 * method is called to populate IOVs suitable for passing to the storage
 * functions.
 */
class GeneratorState
{
  public:
    /**
     * Populate an IOV array
     * @param seq The sequence number (used as a 'selector')
     * @param[out] iov IOV containing pointers to buffers
     *
     * The buffers pointed to in the IOV array remain valid so long as
     * the populateIov() function is not called again on this state
     */
    virtual void populateIov(uint32_t seq, std::vector< lcb_IOV > &iov) = 0;
    virtual ~GeneratorState() {}
};

class SubdocSpec
{
  public:
    std::string path;
    std::string value;
    bool mutate;

    SubdocSpec() : path(""), value(""), mutate(false) {}
};

class SubdocGeneratorState
{
  public:
    /**
     * Populates subdocument command specifications
     * @param seq the sequence number of the current command
     * @param[in,out] specs container to hold the actual spec array.
     *  The spec array must have already been properly pre-sized.
     */
    virtual void populateLookup(uint32_t seq, std::vector< SubdocSpec > &specs) = 0;
    virtual void populateMutate(uint32_t seq, std::vector< SubdocSpec > &specs) = 0;
    virtual ~SubdocGeneratorState() {}
};

class DocGeneratorBase
{
  public:
    /**
     * Create the per-thread state for generating documents
     * @param total_gens Number of total generator threads
     * @param cur_gen The index of the current generator thread
     * @return An opaque state object. This should be deleted by the caller
     */
    virtual GeneratorState *createState(int total_gens, int cur_gen) const = 0;
    virtual SubdocGeneratorState *createSubdocState(int, int) const
    {
        return NULL;
    }
    virtual ~DocGeneratorBase() {}
};

static const char alphabet[] = "0123456789 abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

static void random_fill(std::string &str, int level)
{
    if (level > 1) {
        for (size_t ii = 0; ii < str.size(); ii++) {
            str[ii] = rand() % (0x7e - 0x20) + 0x20; // printable
            switch (str[ii]) {
                case 0x5c: // backslash
                case 0x22: // double quote
                    str[ii]++;
                    break;
            }
        }
    } else if (level > 0) {
        for (size_t ii = 0; ii < str.size(); ii++) {
            str[ii] = alphabet[rand() % (sizeof(alphabet) - 1)];
        }
    }
}

/**
 * Generator class for raw objects. This contains a fixed buffer and will
 * simply vary in how 'long' the buffer is
 */
class RawDocGenerator : public DocGeneratorBase
{
  public:
    /**
     * Generate graded sizes based on a min,max specification. This allows us
     * to be more efficient by cutting corners on how 'random' the sizes
     * actually are. Rather than generating a 'random' size each time we need
     * a document, we split the range into a set of potential sizes (which are
     * also evenly distributed) and cycle between them.
     *
     * @param minsz Smallest desired size
     * @param maxsz Largest desired size
     * @param granularity How many grades to produce
     * @return A vector of sizes which fall between the range
     */
    static std::vector< size_t > gen_graded_sizes(uint32_t minsz, uint32_t maxsz, int grades = 10)
    {
        std::vector< size_t > ret;

        size_t diff = maxsz - minsz;
        size_t factor = diff / grades;
        if (factor == 0 || minsz == maxsz) {
            ret.push_back(maxsz);
        } else {
            for (int ii = 0; ii < grades + 1; ii++) {
                size_t size = minsz + (factor * ii);
                ret.push_back(size);
            }
        }
        return ret;
    }

    RawDocGenerator(uint32_t minsz, uint32_t maxsz, int rnd) : m_sizes(gen_graded_sizes(minsz, maxsz))
    {
        // Populate the buffer to its capacity
        m_buf.insert(0, maxsz, '#');
        if (rnd) {
            random_fill(m_buf, rnd);
        }
    }

    class MyState : public GeneratorState
    {
      public:
        MyState(const RawDocGenerator *parent) : m_parent(parent) {}

        const RawDocGenerator *m_parent;
        void populateIov(uint32_t seq, std::vector< lcb_IOV > &iov_out)
        {
            m_parent->populateIov(seq, iov_out);
        }
    };

    GeneratorState *createState(int, int) const
    {
        return new MyState(this);
    }

  private:
    void populateIov(uint32_t seq, std::vector< lcb_IOV > &iov_out) const
    {
        iov_out.resize(1);
        size_t cursz = m_sizes[seq % m_sizes.size()];
        iov_out[0].iov_base = const_cast< char * >(m_buf.c_str());
        iov_out[0].iov_len = cursz;
    }
    std::string m_buf;
    std::vector< size_t > m_sizes;
};

/**
 * This 'generator' ignores sizes and generates documents as they are received
 * from premade buffers
 */
class PresetDocGenerator : public DocGeneratorBase
{
  public:
    /**
     * @param inputs List of fixed inputs to use
     */
    PresetDocGenerator(const std::vector< std::string > &inputs) : m_bufs(inputs) {}

    class MyState : public GeneratorState
    {
      public:
        MyState(const PresetDocGenerator *parent) : m_parent(parent) {}

        void populateIov(uint32_t seq, std::vector< lcb_IOV > &iov_out)
        {
            m_parent->populateIov(seq, iov_out);
        }

      private:
        const PresetDocGenerator *m_parent;
    };

    GeneratorState *createState(int, int) const
    {
        return new MyState(this);
    }

  protected:
    PresetDocGenerator() {}

    void populateIov(uint32_t seq, std::vector< lcb_IOV > &iov_out) const
    {
        iov_out.resize(1);
        const std::string &s = m_bufs[seq % m_bufs.size()];
        iov_out[0].iov_base = const_cast< char * >(s.c_str());
        iov_out[0].iov_len = s.size();
    }
    std::vector< std::string > m_bufs;
};

// This is the same as the normal document generator, except we generate
// the JSON first
class JsonDocGenerator : public PresetDocGenerator
{
  private:
    struct Doc {
        std::string m_doc;
        class Field
        {
          public:
            Field(const std::string &n, std::string &v) : m_name(n), m_value(v) {}
            const std::string &name() const
            {
                return m_name;
            }
            const std::string &value() const
            {
                return m_value;
            }

          private:
            std::string m_name;
            std::string m_value;
        };
        std::vector< Field > m_fields;
    };
    std::vector< Doc > m_docs;

  public:
    /**
     * @param minsz Minimum JSON document size
     * @param maxsz Maximum JSON document size
     */
    JsonDocGenerator(uint32_t minsz, uint32_t maxsz, int rnd)
    {
        genDocuments(minsz, maxsz, m_docs, rnd);
        for (size_t ii = 0; ii < m_docs.size(); ++ii) {
            m_bufs.push_back(m_docs[ii].m_doc);
        }
    }

    static void genDocuments(uint32_t minsz, uint32_t maxsz, std::vector< std::string > &out, int rnd)
    {
        std::vector< Doc > docs;
        genDocuments(minsz, maxsz, docs, rnd);
        for (size_t ii = 0; ii < docs.size(); ++ii) {
            out.push_back(docs[ii].m_doc);
        }
    }

  private:
    /**
     * Helper method to decrease the original size by a given amount.
     *
     * This also ensures the number never reaches below 0.
     *
     * @param[out] orig Pointer to original size
     * @param toDecr number by which to decrement
     */
    static void decrSize(int *orig, size_t toDecr)
    {
        *orig = std::max(0, *orig - static_cast< int >(toDecr));
    }

    static void genDocuments(uint32_t minsz, uint32_t maxsz, std::vector< Doc > &out, int rnd)
    {
        std::vector< size_t > sizes = RawDocGenerator::gen_graded_sizes(minsz, maxsz);
        for (std::vector< size_t >::iterator ii = sizes.begin(); ii != sizes.end(); ++ii) {
            out.push_back(generate(*ii, rnd));
        }
    }

    /**
     * Generates a "JSON" document of a given size. In order to remain
     * more or less in-tune with common document sizes, field names will be
     * "Field_$incr" and values will be evenly distributed as fixed 16 byte
     * strings. (See JSON_VALUE_SIZE)
     */
    static Doc generate(int docsize, int rnd)
    {
        int counter = 0;
        char keybuf[128] = {0};
        Json::Value root(Json::objectValue);
        Json::FastWriter writer;
        Doc ret;

        while (docsize > 0) {
            decrSize(&docsize, sprintf(keybuf, "Field_%d", ++counter) + 3);
            size_t valsize = std::min(JSON_VALUE_SIZE, docsize);
            if (!valsize) {
                valsize = 1;
            }
            std::string value(valsize, '*');
            if (rnd) {
                random_fill(value, rnd);
            }
            decrSize(&docsize, valsize + 3);
            root[keybuf] = value;
            value = '"' + value;
            value += '"';
            ret.m_fields.push_back(Doc::Field(keybuf, value));
        }
        ret.m_doc = writer.write(root);
        return ret;
    }

    class SDGenstate : public SubdocGeneratorState
    {
      public:
        SDGenstate(const std::vector< Doc > &docs) : m_pathix(0), m_docs(docs) {}

        void populateLookup(uint32_t seq, std::vector< SubdocSpec > &specs)
        {
            populate(seq, specs, false);
        }
        void populateMutate(uint32_t seq, std::vector< SubdocSpec > &specs)
        {
            populate(seq, specs, true);
        }

      private:
        void populate(uint32_t seq, std::vector< SubdocSpec > &specs, bool mutate)
        {
            const Doc &d = doc(seq);
            specs.resize(std::min(d.m_fields.size(), specs.size()));
            for (size_t ii = 0; ii < d.m_fields.size() && ii < specs.size(); ++ii) {
                const Doc::Field &f = d.m_fields[m_pathix++ % d.m_fields.size()];
                SubdocSpec &cur_spec = specs[ii];
                cur_spec.path = f.name();
                cur_spec.mutate = false;
                if (mutate) {
                    cur_spec.value = f.value();
                    cur_spec.mutate = true;
                }
            }
        }

        const Doc &doc(uint32_t seq) const
        {
            return m_docs[seq % m_docs.size()];
        }
        size_t m_pathix;
        const std::vector< Doc > &m_docs;
    };

  public:
    virtual SubdocGeneratorState *createSubdocState(int, int) const
    {
        return new SDGenstate(m_docs);
    }
};

struct TemplateSpec {
    std::string term;
    unsigned minval;
    unsigned maxval;
    bool sequential;
};

/**
 * Generate documents based on placeholder values. Each document (JSON or not)
 * may have one or more special replacement texts which can be substituted
 * with a random number
 */
class PlaceholderDocGenerator : public DocGeneratorBase
{
  public:
    /**
     * @param specs Placeholder specifications
     * @param inputs Documents to use for replacements
     */
    PlaceholderDocGenerator(const std::vector< std::string > &inputs, const std::vector< TemplateSpec > &specs)
    {

        initMatches(specs, inputs);
    }

  protected:
    PlaceholderDocGenerator() {}

    /**
     * Really belongs in constructor, but is decoupled for subclasses
     */
    void initMatches(const std::vector< TemplateSpec > &specs, const std::vector< std::string > &inputs)
    {

        using namespace Placeholders;

        for (std::vector< TemplateSpec >::const_iterator ii = specs.begin(); ii != specs.end(); ++ii) {
            Placeholders::Spec cur(ii->term, ii->minval, ii->maxval, ii->sequential);
            pl_specs.push_back(cur);
        }

        for (std::vector< std::string >::const_iterator ii = inputs.begin(); ii != inputs.end(); ++ii) {
            matches.push_back(new DocumentMatches(*ii, pl_specs));
        }
    }

  public:
    GeneratorState *createState(int total, int cur) const
    {
        return new MyState(this, total, cur);
    }

  private:
    class MyState : public GeneratorState
    {
      public:
        MyState(const PlaceholderDocGenerator *parent, int total, int cur) : m_parent(parent)
        {
            using namespace Placeholders;
            for (size_t ii = 0; ii < parent->matches.size(); ii++) {
                m_substs.push_back(Substitutions(parent->matches[ii], total, cur));
            }
            m_bufs.resize(m_substs.size());
        }

        void populateIov(uint32_t seq, std::vector< lcb_IOV > &iov_out)
        {
            using namespace Placeholders;
            size_t ix = seq % m_substs.size();
            Substitutions &subst = m_substs[ix];
            Substitutions::Backbuffer &buf = m_bufs[ix];
            subst.makeIovs(iov_out, buf);
        }

        std::vector< Placeholders::Substitutions::Backbuffer > m_bufs;
        std::vector< Placeholders::Substitutions > m_substs;
        const PlaceholderDocGenerator *m_parent;
    };

    // Match object for each doc
    std::vector< Placeholders::DocumentMatches * > matches;
    std::vector< Placeholders::Spec > pl_specs;
};

/**
 * Generate documents based on JSON fields. This adds on top of the normal
 * document generator in that JSON paths are used and explicit placeholders
 * are not required
 */
class PlaceholderJsonGenerator : public PlaceholderDocGenerator
{
  public:
    /**
     * @param specs List of specs. The term in the spec refers to the field
     * which is to be replaced, not the placeholder text
     * @param documents The documents to operate on
     */
    PlaceholderJsonGenerator(const std::vector< std::string > &documents, const std::vector< TemplateSpec > &specs)
    {

        initJsonPlaceholders(specs, documents);
    }

    /**
     * Generate the documents, and then generate specs for them
     * @param specs The specs to use
     * @param minsz Minimum document size
     * @param maxsz Maximum document size
     */
    PlaceholderJsonGenerator(uint32_t minsz, uint32_t maxsz, const std::vector< TemplateSpec > &specs, int rnd)
    {

        std::vector< std::string > jsondocs;
        JsonDocGenerator::genDocuments(minsz, maxsz, jsondocs, rnd);
        initJsonPlaceholders(specs, jsondocs);
    }

  private:
    void initJsonPlaceholders(const std::vector< TemplateSpec > &specs, const std::vector< std::string > &documents)
    {

        int serial = 0;
        Json::Reader reader;
        Json::FastWriter writer;
        std::vector< TemplateSpec > new_specs;
        std::vector< std::string > new_docs;

        for (size_t ii = 0; ii < documents.size(); ii++) {
            Json::Value root;
            if (!reader.parse(documents[ii], root)) {
                throw std::runtime_error("Couldn't parse one or more documents!");
            }

            for (size_t jj = 0; jj < specs.size(); ++jj) {
                char buf[64];
                const TemplateSpec &spec = specs[jj];
                sprintf(buf, "$__pillowfight_%d", serial++);
                root[spec.term] = buf;
                TemplateSpec new_spec = spec;

                std::string replace_term(buf);
                replace_term = '"' + replace_term + '"';
                new_spec.term = replace_term;
                new_specs.push_back(new_spec);
            }
            new_docs.push_back(writer.write(root));
        }
        initMatches(new_specs, new_docs);
    }
};

} // namespace Pillowfight
