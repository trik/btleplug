package com.nonpolynomial.btleplug.droidplug

class ConnectionRequest(manager: PeripheralManager) : Request(manager) {
    override fun toString(): String {
        return "Connection request"
    }
}
