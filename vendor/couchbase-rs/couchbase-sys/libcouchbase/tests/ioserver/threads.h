/**
 * @file
 * Simple cross-platform thread abstraction
 */
#ifndef _WIN32
#include <pthread.h>
#endif

class Thread
{
  public:
    typedef void (*StartFunc)(void *);
    Thread(StartFunc, void *);
    ~Thread();
    void close();
    void join();
    void doRun()
    {
        fn(fnparam);
    }

  private:
    StartFunc fn;
    void *fnparam;
    bool initialized;
#ifdef _WIN32
    HANDLE hThread;
#else
    pthread_t thr;
#endif
};

class Condvar;
class Mutex
{
  public:
    Mutex();
    ~Mutex();
    void lock();
    void unlock();
    bool tryLock();
    void close();

  private:
    friend class Condvar;
    bool initialized;
#ifdef _WIN32
    CRITICAL_SECTION cs;
#else
    pthread_mutex_t mutex;
#endif
};

class Condvar
{
  public:
    Condvar();
    ~Condvar();
    void signal();
    void wait(Mutex &);
    void close();

  private:
    bool initialized;
#ifdef _WIN32
    CONDITION_VARIABLE cv;
#else
    pthread_cond_t cond;
#endif
};
