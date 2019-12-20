/* -*- Mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */
/*
 *     Copyright 2015-2019 Couchbase, Inc.
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

#ifndef CBC_PILLOWFIGHT_DOCGEN_H
#define CBC_PILLOWFIGHT_DOCGEN_H

#include <string>
#include <vector>
#include <algorithm>
#include "seqgen.h"
#include "loc.h"

namespace Pillowfight
{
namespace Placeholders
{

/**
 * Placeholder specification.
 * This contains information about a single placeholder's input specification.
 */
class Spec
{
  public:
    /**
     * @param s placeholder string
     * @param minval_ minimum value for replacement
     * @param maxval_ maxmimum value for replacement
     * @param sequential_ if replacement should be done sequentially
     */
    Spec(const std::string &s, unsigned minval, unsigned maxval, unsigned sequential)
        : m_term(s), m_minval(minval), m_maxval(maxval), m_sequential(sequential)
    {
    }

    SeqGenerator *createSeqgen(int total, int cur) const
    {
        if (m_sequential) {
            return new SeqGenerator(m_minval, m_maxval, total, cur);
        } else {
            return new SeqGenerator(m_minval, m_maxval);
        }
    }

    const std::string &term() const
    {
        return m_term;
    }

  private:
    std::string m_term; // Placeholder string to search for
    unsigned m_minval;
    unsigned m_maxval;
    bool m_sequential;
};

/**
 * Mapping of a placeholder specification as it relates to a given template
 * document.
 *
 * This class can be copied since we don't have pointer references within the
 * match itself.
 */
class Match
{
  public:
    // Size of the placeholder (derived from spec)
    size_t size() const
    {
        return m_placeholder->term().size();
    }
    size_t offset() const
    {
        return m_offset;
    }
    const Spec &spec() const
    {
        return *m_placeholder;
    }

    template < class Tin, class Tres > static void find(const std::string &base, const Tin &specs, Tres &results)
    {

        for (size_t ii = 0; ii < specs.size(); ii++) {
            const Spec &pl = specs[ii];
            size_t findpos = base.find(pl.term());
            if (findpos != std::string::npos) {
                results.push_back(Match(&pl, findpos));
            }
        }
        std::sort(results.begin(), results.end(), compare);
    }

  private:
    // Actual placeholder containing the information
    const Spec *m_placeholder;

    // Offset into the document in which the placeholder text begins
    size_t m_offset;
    Match(const Spec *spec_, size_t off) : m_placeholder(spec_), m_offset(off) {}

    static bool compare(const Match &a, const Match &b)
    {
        return a.m_offset < b.m_offset;
    }
};

/**
 * A document with its relevant placeholders. This contains the (constant)
 * document string along with its placeholder information.
 *
 * The document is split into fragments, where some fragments are constant
 * and some are empty and have DocPlaceholder objects mapped to them.
 */
class DocumentMatches
{
  public:
    DocumentMatches(const std::string &original, const std::vector< Spec > &placeholders) : m_base(original)
    {

        Match::find(m_base, placeholders, m_matches);

        const Loc baseloc(m_base.c_str(), m_base.size());
        m_fragments.push_back(baseloc);

        for (size_t ii = 0; ii < m_matches.size(); ii++) {
            Match &dph = m_matches[ii];

            // Location of text to cut out
            Loc dph_loc(m_base.c_str() + dph.offset(), dph.size());

            // Make the last fragment end at the beginning of the current
            // placeholder
            m_fragments.back().rtrim_to(dph_loc);

            // Add the replacement index (current index)
            m_matchix_to_fragix.push_back(m_fragments.size());

            // Add the empty fragment as a Loc to represent the placeholder
            m_fragments.push_back(Loc());

            // Make the next segment contain the rest of the document.
            // If there are more placeholders, then this fragment is truncated
            Loc next_seg;
            next_seg.begin_at_end(baseloc, dph_loc, Loc::NO_OVERLAP);
            m_fragments.push_back(next_seg);
        }
    }

    const std::vector< Match > &matches() const
    {
        return m_matches;
    }

  private:
    DocumentMatches(const DocumentMatches &other);

    friend class Substitutions;
    std::string m_base;             // Base document text
    std::vector< Loc > m_fragments; // Fragments of the document
    std::vector< Match > m_matches;
    std::vector< int > m_matchix_to_fragix; // Mapping of matches to IOV indexes
};

class Substitutions
{
  public:
    // Backing buffer for data
    typedef std::vector< std::string > Backbuffer;

    Substitutions(const DocumentMatches *matches, int total, int cur) : m_matches(matches)
    {
        for (size_t ii = 0; ii < m_matches->m_matches.size(); ii++) {
            const Match &m = m_matches->m_matches[ii];
            SeqGenerator *gen = m.spec().createSeqgen(total, cur);
            m_generators.resize(ii + 1);
            m_generators[ii] = gen;
        }
        for (size_t ii = 0; ii < m_matches->m_fragments.size(); ii++) {
            m_iovs.push_back(m_matches->m_fragments[ii].to_iov());
        }
    }

    /**
     * Create the IOVs necessary for assembling the document
     * @param[out] iovs Vector to contain the output IOVs
     * @param[out] backbuf backing buffers for substitutions
     */
    void makeIovs(std::vector< lcb_IOV > &iovs, Backbuffer &backbuf)
    {
        iovs = m_iovs;
        backbuf.resize(m_matches->matches().size());

        for (size_t ii = 0; ii < m_matches->matches().size(); ii++) {
            char buf[64];
            std::string &output_str = backbuf[ii];
            lcb_IOV &output_iov = iovs[m_matches->m_matchix_to_fragix[ii]];
            SeqGenerator *gen = m_generators[ii];

            // Get the number
            uint32_t cur = gen->next();

            sprintf(buf, "%u", cur);
            output_str.assign(buf);
            output_iov.iov_base = const_cast< char * >(output_str.c_str());
            output_iov.iov_len = output_str.size();
        }
    }

  private:
    const DocumentMatches *m_matches;
    // Array of generators, one for each Match index
    std::vector< SeqGenerator * > m_generators;
    std::vector< lcb_IOV > m_iovs;
};

} // namespace Placeholders
} // namespace Pillowfight

#endif
