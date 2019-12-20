#define NOMINMAX // for win32, use std::min rather than min
#include <gtest/gtest.h>
#include <rdb/rope.h>
#include <algorithm>

class RdbAllocator
{
  public:
    rdb_ALLOCATOR *_inner;
    rdb_ROPESEG *alloc(size_t n)
    {
        return _inner->s_alloc(_inner, n);
    }
    rdb_ROPESEG *realloc(rdb_ROPESEG *prev, size_t n)
    {
        return _inner->s_realloc(_inner, prev, n);
    }
    void reserve(rdb_ROPEBUF *buf, size_t cap)
    {
        _inner->r_reserve(_inner, buf, cap);
    }
    void free(rdb_ROPESEG *seg)
    {
        _inner->s_release(_inner, seg);
    }
    void release()
    {
        _inner->a_release(_inner);
    }

    RdbAllocator(rdb_ALLOCATOR *inner)
    {
        _inner = inner;
    }
};

struct IORope : public rdb_IOROPE {
    IORope(rdb_ALLOCATOR *allocator)
    {
        rdb_init(this, allocator);
        rdsize = 256;
    }

    IORope()
    {
        rdb_init(this, rdb_bigalloc_new());
        rdsize = 256;
    }

    ~IORope()
    {
        rdb_cleanup(this);
    }

    IORope(const IORope &);

    std::string stlstr(size_t n)
    {
        char *buf = new char[n];
        rdb_copyread(this, buf, n);
        std::string rv(buf, n);
        delete[] buf;
        return rv;
    }

    size_t usedSize() const
    {
        return recvd.nused;
    }

    void feed(const std::string &s)
    {
        size_t n_fed = 0;
        nb_IOV iov[32];

        while (n_fed < s.size()) {
            unsigned niov = rdb_rdstart(this, iov, 32);
            unsigned cur_nfed = 0;

            for (unsigned ii = 0; ii < niov && n_fed < s.size(); ii++) {
                const char *frag = s.data() + n_fed;
                nb_IOV *curiov = iov + ii;
                // on win32 iov_len is not a size_t
                unsigned to_copy = std::min(s.size() - n_fed, (size_t)curiov->iov_len);
                memcpy(curiov->iov_base, frag, to_copy);
                n_fed += to_copy;
                cur_nfed += to_copy;
            }
            rdb_rdend(this, cur_nfed);
        }
    }

    void feed(const char *s)
    {
        feed(std::string(s));
    }
};

struct ReadPacket {
    std::vector< rdb_ROPESEG * > segments;
    std::vector< nb_IOV > iovs;

    ReadPacket(nb_IOV *iov, rdb_ROPESEG **segs, unsigned n)
    {
        segments.reserve(n);
        iovs.reserve(n);

        // Insert them.
        segments.insert(segments.begin(), segs, segs + n);
        iovs.insert(iovs.begin(), iov, iov + n);
    }

    ReadPacket(rdb_IOROPE *ior, unsigned nb)
    {
        segments.resize(2);
        iovs.resize(2);
        unsigned niov;

        while (true) {
            niov = rdb_refread_ex(ior, &iovs[0], &segments[0], iovs.size(), nb);
            if (niov != (unsigned)-1) {
                iovs.resize(niov);
                segments.resize(niov);
                break;
            }

            iovs.resize(iovs.size() * 2);
            segments.resize(segments.size() * 2);
        }
    }

    void refSegment(unsigned ix)
    {
        rdb_seg_ref(segments[ix]);
    }

    std::string asString()
    {
        std::string s;
        for (size_t ii = 0; ii < iovs.size(); ii++) {
            nb_IOV *cur = &iovs[ii];
            s.append((const char *)cur->iov_base, cur->iov_len);
        }
        return s;
    }

    void unrefSegment(unsigned ix)
    {
        rdb_seg_unref(segments[ix]);
    }
};
