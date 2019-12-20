#ifndef _WIN32
#include "threads.h"
#include <cstdlib>
#include <unistd.h>
#include <pthread.h>

extern "C" {
static void *startfunc(void *arg)
{
    Thread *thr = reinterpret_cast< Thread * >(arg);
    thr->doRun();
    return NULL;
}
}

Thread::Thread(StartFunc startfn, void *arg)
{
    this->fn = startfn;
    this->fnparam = arg;
    initialized = true;
    pthread_create(&thr, NULL, startfunc, this);
}

void Thread::close()
{
    if (!initialized) {
        return;
    }
    join();
    initialized = false;
}

Thread::~Thread()
{
    close();
}

void Thread::join()
{
    void *res;
    pthread_join(thr, &res);
}

// Mutex:
Mutex::Mutex()
{
    pthread_mutex_init(&mutex, NULL);
    initialized = true;
}

void Mutex::lock()
{
    pthread_mutex_lock(&mutex);
}

bool Mutex::tryLock()
{
    return pthread_mutex_trylock(&mutex) == 0;
}

void Mutex::unlock()
{
    pthread_mutex_unlock(&mutex);
}

void Mutex::close()
{
    if (initialized) {
        initialized = false;
        pthread_mutex_destroy(&mutex);
    }
}

Mutex::~Mutex()
{
    close();
}

// Condvar
Condvar::Condvar()
{
    pthread_cond_init(&cond, NULL);
    initialized = true;
}

void Condvar::wait(Mutex &mutex)
{
    pthread_cond_wait(&cond, &mutex.mutex);
}

void Condvar::signal()
{
    pthread_cond_signal(&cond);
}

void Condvar::close()
{
    if (initialized) {
        pthread_cond_destroy(&cond);
        initialized = false;
    }
}

Condvar::~Condvar()
{
    close();
}
#endif
