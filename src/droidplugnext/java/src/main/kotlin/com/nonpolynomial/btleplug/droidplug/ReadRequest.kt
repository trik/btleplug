package com.nonpolynomial.btleplug.droidplug

import java.util.UUID

class ReadRequest(manager: PeripheralManager, val service: UUID, val characteristic: UUID) : Request(manager) {
    var value: ByteArray? = null

    override fun toString(): String {
        return "Read request service $service characteristic $characteristic"
    }
}
