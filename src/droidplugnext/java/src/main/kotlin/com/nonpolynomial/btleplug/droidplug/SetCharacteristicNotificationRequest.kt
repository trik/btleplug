package com.nonpolynomial.btleplug.droidplug

import java.util.UUID

class SetCharacteristicNotificationRequest(manager: PeripheralManager, val service: UUID, val characteristic: UUID, val enable: Boolean) : Request(manager) {
    override fun toString(): String {
        return "Set notification service $service characteristic $characteristic enable $enable"
    }
}
