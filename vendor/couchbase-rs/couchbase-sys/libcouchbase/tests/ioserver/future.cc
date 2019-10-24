#include "ioserver.h"
using namespace LCBTest;

Future::Future()
{
    failed = false;
}

void Future::wait()
{
    mutex.lock();
    while (!isDone() && !failed) {
        cond.wait(mutex);
    }
    mutex.unlock();
}

Future::~Future()
{
    mutex.close();
    cond.close();
}

void Future::startUpdate()
{
    mutex.lock();
}

void Future::endUpdate()
{
    if (shouldEnd()) {
        cond.signal();
    }
    mutex.unlock();
}

bool Future::checkDone()
{
    bool ret;
    if (!mutex.tryLock()) {
        return false;
    }
    ret = isDone();
    mutex.unlock();
    return ret;
}
