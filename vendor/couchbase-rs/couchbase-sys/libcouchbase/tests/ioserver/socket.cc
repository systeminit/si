#include "ioserver.h"
using namespace LCBTest;

SockFD::SockFD(int sock)
{
    assert(sock >= 0);
    fd = sock;
    naddr = sizeof(sa_local);
    int rv = getsockname(fd, (struct sockaddr *)&sa_local, &naddr);
    assert(rv == 0);
}

void SockFD::loadRemoteAddr()
{
    socklen_t lentmp = sizeof(sa_remote);
    getpeername(*this, (struct sockaddr *)&sa_remote, &lentmp);
}

void SockFD::close()
{
    if (fd != -1) {
        shutdown(fd, SHUT_RDWR);
        ::closesocket(fd);
        fd = -1;
    }
}

SockFD::~SockFD()
{
    close();
}

SockFD *SockFD::acceptClient()
{
    struct sockaddr_storage newaddr;
    socklen_t newlen = sizeof(newaddr);
    int newsock = accept(*this, (struct sockaddr *)&newaddr, &newlen);
    return new SockFD(newsock);
}

SockFD *SockFD::newListener()
{
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));

    int lsnfd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = inet_addr("127.0.0.1");
    addr.sin_port = 0;
    bind(lsnfd, (struct sockaddr *)&addr, sizeof(addr));
    listen(lsnfd, 5);
    return new SockFD(lsnfd);
}

SockFD *SockFD::newClient(SockFD *server)
{
    int sockfd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    assert(sockfd >= 0);
    int rv = connect(sockfd, (struct sockaddr *)&server->localAddr4(), sizeof(struct sockaddr_in));
    assert(rv == 0);
    return new SockFD(sockfd);
}

#ifndef _WIN32
std::string SockFD::getHostCommon(sockaddr_storage *ss)
{
    struct sockaddr_in *addr = (struct sockaddr_in *)ss;
    char buf[4096];
    inet_ntop(AF_INET, &addr->sin_addr, buf, sizeof(*addr));
    return std::string(buf);
}
#else
std::string SockFD::getHostCommon(sockaddr_storage *ss)
{
    struct sockaddr_in *inaddr = (struct sockaddr_in *)ss;
    char *buf = inet_ntoa(inaddr->sin_addr);
    return std::string(buf);
}
#endif
