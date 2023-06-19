package com.nonpolynomial.btleplug.droidplug

import java.util.UUID

class WriteRequest(manager: PeripheralManager, val service: UUID, val characteristic: UUID, val value: ByteArray, val withoutResponse: Boolean) : Request(manager) {
    override fun toString(): String {
        return "Read request service $service characteristic $characteristic"
    }
}
