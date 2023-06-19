package com.nonpolynomial.btleplug.droidplug

import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothGattService
import android.bluetooth.BluetoothProfile
import android.util.Log
import com.nonpolynomial.btleplug.droidplug.Request
import java.util.UUID

import java.util.concurrent.LinkedBlockingQueue

class PeripheralManager(private val device: BluetoothDevice): Base() {
    companion object {
        private const val tag = "PeripheralManager"
        private val cccdUuid = UUID.fromString("000002902-0000-1000-8000-00805f9b34fb")
    }
    private val lock = Object()
    var connected = false
//        get() = synchronized(lock) { connected }
        private set
    var discovered = false
        private set
    private val requestsQueue = LinkedBlockingQueue<Request>()
    private var processing = false
    private var gatt: BluetoothGatt? = null
    // current request running
    private var request: Request? = null

    fun connect(): Request {
        return ConnectionRequest(this)
    }

    fun disconnect(): Request {
        return DisconnectionRequest(this)
    }

    fun discoverServices(): Request {
        return DiscoverServicesRequest(this)
    }

    fun read(service: UUID, characteristic: UUID): ReadRequest {
        return ReadRequest(this, service, characteristic)
    }

    fun write(service: UUID, characteristic: UUID, value: ByteArray, withoutResponse: Boolean): Request {
        return WriteRequest(this, service, characteristic, value, withoutResponse)
    }

    fun setCharacteristicNotification(service: UUID, characteristic: UUID, enable: Boolean): SetCharacteristicNotificationRequest {
        return SetCharacteristicNotificationRequest(this, service, characteristic, enable)
    }

    fun enqueue(request: Request): Unit {
        Log.d(tag, "Enqueuing a $request")
        requestsQueue.add(request)
        nextRequest()
    }

    fun getServices(): List<Service> {
        val gatt = gatt ?: return emptyList()
        return gatt.services.map { service ->
            Service(
                    service.uuid.toString(),
                    false,
                    service.characteristics.map { characteristic ->
                        Characteristic(
                                characteristic.uuid.toString(),
                                characteristic.properties,
                                characteristic.descriptors.map { descriptor ->
                                    descriptor.uuid.toString()
                                }
                        )
                    }
            )
        }
    }

    @Synchronized
    private fun nextRequest() {
        if (processing || requestsQueue.isEmpty()) {
            return
        }
        processing = true
        val request = requestsQueue.remove()
        val result = when (request) {
            is ConnectionRequest -> process(request)
            is DisconnectionRequest -> process(request)
            is DiscoverServicesRequest -> process(request)
            is ReadRequest -> process(request)
            is WriteRequest -> process(request)
            is SetCharacteristicNotificationRequest -> process(request)
        }
        if (!result) {
            request.notifyFailed(Request.REASON_REQUEST_INVALID)
        }
        processing = false
        nextRequest()
    }

    private fun process(request: ConnectionRequest): Boolean {
        Log.d(tag, "Connecting gatt on $device")
        try {
            device.connectGatt(contextProvider.context, false, gattCallback, BluetoothDevice.TRANSPORT_LE)
        } catch (e: SecurityException) {
            request.notifyFailed(Request.REASON_MISSING_PERMISSIONS)
        }
        return true
    }

    private fun process(request: DisconnectionRequest): Boolean {
        Log.d(tag, "Disconnecting gatt on $device")
        val gatt = gatt ?: return false
        try {
            gatt.close()
        } catch (e: SecurityException) {
            request.notifyFailed(Request.REASON_MISSING_PERMISSIONS)
        }
        return true
    }

    private fun process(request: DiscoverServicesRequest): Boolean {
        Log.d(tag, "Discovering gatt services on $device")
        val gatt = gatt ?: return false
        try {
            return gatt.discoverServices()
        } catch (e: SecurityException) {
            request.notifyFailed(Request.REASON_MISSING_PERMISSIONS)
        }
        return true
    }

    private fun process(request: ReadRequest): Boolean {
        Log.d(tag, "Reading service ${request.service} characteristic ${request.characteristic} on $device")
        val gatt = gatt ?: return false
        with(request) {
            val characteristic = getCharacteristic(service, characteristic) ?: return false
            try {
                return gatt.readCharacteristic(characteristic)
            } catch (e: SecurityException) {
                request.notifyFailed(Request.REASON_MISSING_PERMISSIONS)
            }
        }
        return true
    }

    private fun process(request: WriteRequest): Boolean {
        Log.d(tag, "Writing service ${request.service} characteristic ${request.characteristic} on $device")
        val gatt = gatt ?: return false
        with(request) {
            val characteristic = getCharacteristic(service, characteristic) ?: return false
            try {
                characteristic.value = request.value
                characteristic.writeType = if (request.withoutResponse) BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE else BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT
                val result = gatt.writeCharacteristic(characteristic)
                if (!result) {
                    return false
                }
                if (request.withoutResponse) {
                    request.notifyCompleted()
                }
                return true
            } catch (e: SecurityException) {
                request.notifyFailed(Request.REASON_MISSING_PERMISSIONS)
            }
        }
        return true
    }

    private fun process(request: SetCharacteristicNotificationRequest): Boolean {
        Log.d(tag, "Setting characteristic notification service ${request.service} characteristic ${request.characteristic} enable ${request.enable} on $device")
        val gatt = gatt ?: return false
        with(request) {
            val characteristic = getCharacteristic(service, characteristic) ?: return false
            try {
                return gatt.setCharacteristicNotification(characteristic, enable)
            } catch (e: SecurityException) {
                request.notifyFailed(Request.REASON_MISSING_PERMISSIONS)
            }
        }
        return true
    }

    private fun getCharacteristicCCCD(service: UUID, characteristic: UUID): BluetoothGattDescriptor? {
        return getDescriptor(service, characteristic, UUID.fromString("000002902-0000-1000-8000-00805f9b34fb"))
    }

    private fun getDescriptor(service: UUID, characteristic: UUID, descriptor: UUID): BluetoothGattDescriptor? {
        return getCharacteristic(service, characteristic)?.descriptors?.find { it.uuid == descriptor }
    }

    private fun getCharacteristic(service: UUID, characteristic: UUID): BluetoothGattCharacteristic? {
        return getService(service)?.characteristics?.find { it.uuid == characteristic }
    }

    private fun getService(service: UUID): BluetoothGattService? {
        return gatt?.services?.find { it.uuid == service }
    }

    private val gattCallback = object: BluetoothGattCallback() {
        override fun onConnectionStateChange(gatt: BluetoothGatt?, status: Int, newState: Int) {
            if (gatt == null) {
                return
            }
            val connected = status == BluetoothGatt.GATT_SUCCESS && newState == BluetoothProfile.STATE_CONNECTED
//            synchronized(lock) { connected = newConnected }
            this@PeripheralManager.connected = connected
            when (val request = this@PeripheralManager.request) {
                null -> {}
                is ConnectionRequest -> {
                    request.notify(if (connected) Request.REASON_INVALID_REQUEST else status)
                }
                is DisconnectionRequest -> {
                    request.notify(if (!connected) Request.REASON_INVALID_REQUEST else status)
                }
                else -> request.notifyFailed(Request.REASON_INVALID_REQUEST)
            }
        }

        override fun onServicesDiscovered(gatt: BluetoothGatt?, status: Int) {
            if (gatt == null) {
                return
            }
            val discovered = status == BluetoothGatt.GATT_SUCCESS
            this@PeripheralManager.discovered = discovered
            when (val request = this@PeripheralManager.request) {
                null -> {}
                is DiscoverServicesRequest -> {
                    if (discovered) {
                        request.notifyCompleted()
                    } else {
                        request.notifyFailed(status)
                    }
                }
                else -> request.notifyFailed(Request.REASON_INVALID_REQUEST)
            }
        }

        override fun onCharacteristicRead(gatt: BluetoothGatt?, characteristic: BluetoothGattCharacteristic?, status: Int) {
            if (gatt == null) {
                return
            }
            val read = characteristic != null && status == BluetoothGatt.GATT_SUCCESS
            when (val request = this@PeripheralManager.request) {
                null -> {}
                is ReadRequest -> {
                    if (read) {
                        request.value = characteristic!!.value
                        request.notifyCompleted()
                    } else {
                        request.notifyFailed(status)
                    }
                }
                else -> request.notifyFailed(Request.REASON_INVALID_REQUEST)
            }
        }

        override fun onCharacteristicWrite(gatt: BluetoothGatt?, characteristic: BluetoothGattCharacteristic?, status: Int) {
            if (gatt == null) {
                return
            }
            val written = characteristic != null && status == BluetoothGatt.GATT_SUCCESS
            when (val request = this@PeripheralManager.request) {
                null -> {}
                is WriteRequest -> {
                    if (request.characteristic != characteristic!!.uuid || request.service != characteristic.service.uuid) {
                        request.notifyFailed(Request.REASON_INVALID_CHARACTERISTIC)
                    }
                    if (request.withoutResponse) {
                        request.notifyFailed(Request.REASON_INVALID_REQUEST)
                    } else {
                        if (written) {
                            request.notifyCompleted()
                        } else {
                            request.notifyFailed(Request.REASON_WRITE_FAILED)
                        }
                    }
                }
                else -> request.notifyFailed(Request.REASON_INVALID_REQUEST)
            }
        }

        override fun onDescriptorWrite(gatt: BluetoothGatt?, descriptor: BluetoothGattDescriptor?, status: Int) {
            if (gatt == null) {
                return
            }
            val written = descriptor != null && status == BluetoothGatt.GATT_SUCCESS
            when (val request = this@PeripheralManager.request) {
                null -> {}
                is SetCharacteristicNotificationRequest -> {
                    if (descriptor!!.uuid != cccdUuid || request.characteristic != descriptor.characteristic.uuid || request.service != descriptor.characteristic.service.uuid) {
                        request.notifyFailed(Request.REASON_INVALID_CHARACTERISTIC)
                    }
                    if (written) {
                        request.notifyCompleted()
                    } else {
                        request.notifyFailed(Request.REASON_SET_NOTIFICATION_FAILED)
                    }
                }
                else -> request.notifyFailed(Request.REASON_INVALID_REQUEST)
            }
        }
    }
}
