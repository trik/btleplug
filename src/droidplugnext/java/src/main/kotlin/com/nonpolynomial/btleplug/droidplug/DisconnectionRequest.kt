package com.nonpolynomial.btleplug.droidplug

class DisconnectionRequest(manager: PeripheralManager) : Request(manager) {
    override fun toString(): String {
        return "Disconnection request"
    }
}
