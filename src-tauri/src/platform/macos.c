#include <SystemConfiguration/SystemConfiguration.h>
#include <CoreFoundation/CoreFoundation.h>
#include <dispatch/dispatch.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <string.h>

static _Atomic uint64_t generation = 1;
static pthread_once_t observer_once = PTHREAD_ONCE_INIT;
static SCDynamicStoreRef observer;
static void changed(SCDynamicStoreRef store, CFArrayRef keys, void *info) {
    (void)store; (void)keys; (void)info;
    atomic_fetch_add_explicit(&generation, 1, memory_order_release);
}
static void observe(void) {
    observer = SCDynamicStoreCreate(NULL, CFSTR("Local Chat network changes"), changed, NULL);
    if (!observer) return;
    const void *patterns[] = { CFSTR("State:/Network/Global/IPv4"), CFSTR("State:/Network/Global/IPv6"), CFSTR("State:/Network/Interface/.*/IPv4"), CFSTR("State:/Network/Interface/.*/Link"), CFSTR("State:/Network/Service/.*/IPv4") };
    CFArrayRef array = CFArrayCreate(NULL, patterns, 5, &kCFTypeArrayCallBacks);
    SCDynamicStoreSetNotificationKeys(observer, NULL, array);
    SCDynamicStoreSetDispatchQueue(observer, dispatch_get_global_queue(QOS_CLASS_UTILITY,0));
    CFRelease(array);
}
uint64_t localchat_network_generation(void) {
    pthread_once(&observer_once, observe);
    return atomic_load_explicit(&generation, memory_order_acquire);
}
static CFStringRef string_value(CFDictionaryRef dict, CFStringRef key) {
    const void *value = CFDictionaryGetValue(dict,key);
    return value && CFGetTypeID(value)==CFStringGetTypeID() ? (CFStringRef)value : NULL;
}
int localchat_gateway_hardware(const char *interface, const char *gateway, char *output, size_t capacity) {
    int result = 0;
    SCDynamicStoreRef store = SCDynamicStoreCreate(NULL,CFSTR("Local Chat network evidence"),NULL,NULL);
    if (!store) return 0;
    CFDictionaryRef global = SCDynamicStoreCopyValue(store,CFSTR("State:/Network/Global/IPv4"));
    if (!global || CFGetTypeID(global)!=CFDictionaryGetTypeID()) { if(global)CFRelease(global); CFRelease(store); return 0; }
    CFStringRef service = string_value(global,CFSTR("PrimaryService"));
    CFStringRef physical = string_value(global,CFSTR("PrimaryInterface"));
    char physical_buffer[64];
    if (!service || !physical || !CFStringGetCString(physical,physical_buffer,sizeof(physical_buffer),kCFStringEncodingUTF8) || strcmp(physical_buffer,interface)!=0) goto cleanup;
    CFStringRef key = CFStringCreateWithFormat(NULL,NULL,CFSTR("State:/Network/Service/%@/IPv4"),service);
    CFDictionaryRef state = SCDynamicStoreCopyValue(store,key);
    CFRelease(key);
    if (state && CFGetTypeID(state)==CFDictionaryGetTypeID()) {
        CFStringRef address=string_value(state,CFSTR("ARPResolvedIPAddress"));
        CFStringRef hardware=string_value(state,CFSTR("ARPResolvedHardwareAddress"));
        CFStringRef confirmed=string_value(state,CFSTR("ConfirmedInterfaceName"));
        char address_buffer[64], interface_buffer[64];
        if (address && hardware && confirmed && CFStringGetCString(address,address_buffer,sizeof(address_buffer),kCFStringEncodingUTF8) && strcmp(address_buffer,gateway)==0 && CFStringGetCString(confirmed,interface_buffer,sizeof(interface_buffer),kCFStringEncodingUTF8) && strcmp(interface_buffer,interface)==0) {
            result=CFStringGetCString(hardware,output,capacity,kCFStringEncodingUTF8);
        }
    }
    if(state)CFRelease(state);
cleanup:
    CFRelease(global); CFRelease(store); return result;
}

#include <net/if.h>
#include <sys/socket.h>
#include <netinet/in.h>
int localchat_bind_interface(int descriptor, const char *name) {
    unsigned int index=if_nametoindex(name);
    if (!index) return -1;
    return setsockopt(descriptor,IPPROTO_IP,IP_BOUND_IF,&index,sizeof(index));
}
