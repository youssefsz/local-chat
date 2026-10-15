#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
#include <windows.h>
#include <iphlpapi.h>
#include <stdint.h>

static volatile LONG64 generation = 1;
static INIT_ONCE once = INIT_ONCE_STATIC_INIT;
static HANDLE route_handle, address_handle, interface_handle;
static BOOL ready = FALSE;
static VOID CALLBACK route_changed(PVOID ctx, PMIB_IPFORWARD_ROW2 row, MIB_NOTIFICATION_TYPE type) {
    (void)ctx; (void)row; (void)type; InterlockedIncrement64(&generation);
}
static VOID CALLBACK address_changed(PVOID ctx, PMIB_UNICASTIPADDRESS_ROW row, MIB_NOTIFICATION_TYPE type) {
    (void)ctx; (void)row; (void)type; InterlockedIncrement64(&generation);
}
static VOID CALLBACK interface_changed(PVOID ctx, PMIB_IPINTERFACE_ROW row, MIB_NOTIFICATION_TYPE type) {
    (void)ctx; (void)row; (void)type; InterlockedIncrement64(&generation);
}
static BOOL CALLBACK start(PINIT_ONCE init, PVOID param, PVOID *context) {
    (void)init; (void)param; (void)context;
    ready = NotifyRouteChange2(AF_UNSPEC, route_changed, NULL, FALSE, &route_handle) == NO_ERROR
        && NotifyUnicastIpAddressChange(AF_UNSPEC, address_changed, NULL, FALSE, &address_handle) == NO_ERROR
        && NotifyIpInterfaceChange(AF_UNSPEC, interface_changed, NULL, FALSE, &interface_handle) == NO_ERROR;
    return TRUE;
}
uint64_t localchat_network_generation(void) {
    InitOnceExecuteOnce(&once, start, NULL, NULL);
    return ready ? (uint64_t)InterlockedCompareExchange64(&generation, 0, 0) : 0;
}
int localchat_physical_interface(uint32_t index) {
    MIB_IF_ROW2 row = {0}; row.InterfaceIndex = index;
    if (GetIfEntry2(&row) != NO_ERROR) return 0;
    return row.InterfaceAndOperStatusFlags.HardwareInterface
        && !row.InterfaceAndOperStatusFlags.FilterInterface
        && row.InterfaceAndOperStatusFlags.ConnectorPresent;
}
