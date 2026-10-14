#include <dns_sd.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <poll.h>
#include <net/if.h>
#include <arpa/inet.h>

typedef void (*event_callback)(void *,int,const char *,const char *,const char *,const char *,unsigned short);
struct discovery;
struct record { struct discovery *owner; int used; DNSServiceRef resolve,address; char fullname[1024],peer[65],cert[65]; unsigned short port; };
struct discovery { pthread_t thread; _Atomic int stop; unsigned int interface; unsigned short port; char name[64],service_type[64]; unsigned char txt[512]; unsigned short txt_len; DNSServiceRef connection,registration,browse; struct record records[64]; event_callback callback; void *user; };
static void failure(struct discovery *d,DNSServiceErrorType error) { char text[128];snprintf(text,sizeof(text),"Bonjour discovery error %d",error);d->callback(d->user,2,text,"","","",0); }
static void registration_callback(DNSServiceRef ref,DNSServiceFlags flags,DNSServiceErrorType error,const char *name,const char *type,const char *domain,void *context) { (void)ref;(void)flags;(void)name;(void)type;(void)domain;if(error)failure(context,error); }
static void address_callback(DNSServiceRef ref,DNSServiceFlags flags,uint32_t interface,DNSServiceErrorType error,const char *hostname,const struct sockaddr *address,uint32_t ttl,void *context) {
    (void)ref;(void)hostname;(void)ttl;struct record *r=context;
    if(error){failure(r->owner,error);return;}
    if(!(flags&kDNSServiceFlagsAdd)||interface!=r->owner->interface||!address||address->sa_family!=AF_INET)return;
    char ip[INET_ADDRSTRLEN];if(!inet_ntop(AF_INET,&((const struct sockaddr_in *)address)->sin_addr,ip,sizeof(ip)))return;
    r->owner->callback(r->owner->user,0,r->fullname,r->peer,r->cert,ip,r->port);
}
static void resolve_callback(DNSServiceRef ref,DNSServiceFlags flags,uint32_t interface,DNSServiceErrorType error,const char *fullname,const char *hosttarget,uint16_t port,uint16_t txt_len,const unsigned char *txt,void *context) {
    (void)ref;(void)flags;struct record *r=context;
    if(error){failure(r->owner,error);return;}
    uint8_t protocol_len=0,version_len=0,peer_len=0,cert_len=0;
    const char *protocol=TXTRecordGetValuePtr(txt_len,txt,"protocol",&protocol_len);
    const char *version=TXTRecordGetValuePtr(txt_len,txt,"version",&version_len);
    const char *peer=TXTRecordGetValuePtr(txt_len,txt,"peer",&peer_len);
    const char *cert=TXTRecordGetValuePtr(txt_len,txt,"cert",&cert_len);
    if(!protocol||protocol_len!=9||memcmp(protocol,"localchat",9)||!version||version_len!=1||*version!='1'||!peer||peer_len!=64||!cert||cert_len!=64)return;
    memcpy(r->peer,peer,64);r->peer[64]=0;memcpy(r->cert,cert,64);r->cert[64]=0;r->port=ntohs(port);
    snprintf(r->fullname,sizeof(r->fullname),"%s",fullname);
    if(r->address)DNSServiceRefDeallocate(r->address);
    r->address=r->owner->connection;
    error=DNSServiceGetAddrInfo(&r->address,kDNSServiceFlagsShareConnection,interface,kDNSServiceProtocol_IPv4,hosttarget,address_callback,r);
    if(error){r->address=NULL;failure(r->owner,error);}
}
static void clear_record(struct record *r){if(r->resolve)DNSServiceRefDeallocate(r->resolve);if(r->address)DNSServiceRefDeallocate(r->address);memset(r,0,sizeof(*r));}
static void browse_callback(DNSServiceRef ref,DNSServiceFlags flags,uint32_t interface,DNSServiceErrorType error,const char *name,const char *type,const char *domain,void *context) {
    (void)ref;struct discovery *d=context;if(error){failure(d,error);return;}if(interface!=d->interface)return;
    char fullname[1024];if(DNSServiceConstructFullName(fullname,name,type,domain))return;
    struct record *r=NULL,*empty=NULL;
    for(size_t i=0;i<64;i++){if(d->records[i].used&&!strcmp(d->records[i].fullname,fullname)){r=&d->records[i];break;}if(!d->records[i].used&&!empty)empty=&d->records[i];}
    if(!(flags&kDNSServiceFlagsAdd)){d->callback(d->user,1,fullname,"","","",0);if(r)clear_record(r);return;}
    if(r)clear_record(r);else r=empty;if(!r)return;
    r->used=1;r->owner=d;snprintf(r->fullname,sizeof(r->fullname),"%s",fullname);r->resolve=d->connection;
    error=DNSServiceResolve(&r->resolve,kDNSServiceFlagsShareConnection,interface,name,type,domain,resolve_callback,r);
    if(error){r->resolve=NULL;failure(d,error);}
}
static void *run(void *context){
    struct discovery *d=context;DNSServiceErrorType error=DNSServiceCreateConnection(&d->connection);
    if(error){failure(d,error);return NULL;}
    d->registration=d->connection;
    error=DNSServiceRegister(&d->registration,kDNSServiceFlagsShareConnection,d->interface,d->name,d->service_type,"local.",NULL,htons(d->port),d->txt_len,d->txt,registration_callback,d);
    if(error){d->registration=NULL;failure(d,error);goto cleanup;}
    d->browse=d->connection;
    error=DNSServiceBrowse(&d->browse,kDNSServiceFlagsShareConnection,d->interface,d->service_type,"local.",browse_callback,d);
    if(error){d->browse=NULL;failure(d,error);goto cleanup;}
    int descriptor=DNSServiceRefSockFD(d->connection);
    while(!atomic_load_explicit(&d->stop,memory_order_acquire)){
        struct pollfd item={.fd=descriptor,.events=POLLIN};int status=poll(&item,1,200);
        if(status>0&&(item.revents&POLLIN)){error=DNSServiceProcessResult(d->connection);if(error){failure(d,error);break;}}
        else if(status<0)break;
    }
cleanup:
    for(size_t i=0;i<64;i++)if(d->records[i].used)clear_record(&d->records[i]);
    if(d->registration)DNSServiceRefDeallocate(d->registration);if(d->browse)DNSServiceRefDeallocate(d->browse);if(d->connection)DNSServiceRefDeallocate(d->connection);return NULL;
}
void *localchat_bonjour_start(const char *interface,const char *name,const char *service_type,unsigned short port,const unsigned char *txt,unsigned short txt_len,event_callback callback,void *user){
    if(txt_len>512)return NULL;unsigned int index=if_nametoindex(interface);if(!index)return NULL;
    struct discovery *d=calloc(1,sizeof(*d));if(!d)return NULL;d->interface=index;snprintf(d->service_type,sizeof(d->service_type),"%s",service_type);d->port=port;d->txt_len=txt_len;memcpy(d->txt,txt,txt_len);snprintf(d->name,sizeof(d->name),"%s",name);d->callback=callback;d->user=user;
    if(pthread_create(&d->thread,NULL,run,d)){free(d);return NULL;}return d;
}
void localchat_bonjour_stop(void *context){struct discovery *d=context;if(!d)return;atomic_store_explicit(&d->stop,1,memory_order_release);pthread_join(d->thread,NULL);free(d);}
