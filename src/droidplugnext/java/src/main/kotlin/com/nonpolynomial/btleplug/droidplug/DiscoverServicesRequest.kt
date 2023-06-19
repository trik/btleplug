package com.nonpolynomial.btleplug.droidplug

class DiscoverServicesRequest(manager: PeripheralManager) : Request(manager) {
    override fun toString(): String {
        return "Discover services request"
    }
}
