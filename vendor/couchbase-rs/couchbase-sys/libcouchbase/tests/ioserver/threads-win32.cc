#ifdef _WIN32
#include <cassert>
#include <windows.h>
#include <process.h>
#include "threads.h"

extern "C" {
unsigned int __stdcall startfunc(void *param)
{
    Thread *thr = reinterpret_cast< Thread * >(param);
    thr->doRun();
    return 0;
}
}

Thread::Thread(StartFunc thrfn, void *param)
{
    fn = thrfn;
    fnparam = param;
    uintptr_t rv;
    rv = _beginthreadex(NULL, 0, startfunc, this, 0, NULL);
    assert(rv);
    hThread = (HANDLE)rv;
    initialized = true;
}

void Thread::join()
{
    WaitForSingleObject(hThread, INFINITE);
}

void Thread::close()
{
    if (initialized) {
        join();
        CloseHandle(hThread);
        initialized = false;
    }
}

Thread::~Thread()
{
    close();
}

Mutex::Mutex()
{
    InitializeCriticalSection(&cs);
    initialized = true;
}

Mutex::~Mutex()
{
    close();
}

void Mutex::close()
{
    if (initialized) {
        DeleteCriticalSection(&cs);
        initialized = false;
    }
}

void Mutex::lock()
{
    EnterCriticalSection(&cs);
}

bool Mutex::tryLock()
{
    return TryEnterCriticalSection(&cs) == TRUE;
}

void Mutex::unlock()
{
    LeaveCriticalSection(&cs);
}

Condvar::Condvar()
{
    InitializeConditionVariable(&cv);
    initialized = true;
}

void Condvar::close()
{
    initialized = false;
}

Condvar::~Condvar()
{
    close();
}

void Condvar::signal()
{
    WakeConditionVariable(&cv);
}

void Condvar::wait(Mutex &mutex)
{
    SleepConditionVariableCS(&cv, &mutex.cs, INFINITE);
}
#endif
