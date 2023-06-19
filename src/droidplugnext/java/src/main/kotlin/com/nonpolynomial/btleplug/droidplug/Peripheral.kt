package com.nonpolynomial.btleplug.droidplug

import android.bluetooth.BluetoothDevice
import android.util.Log
import java.util.UUID
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors

@Suppress("unused")
class Peripheral(address: String) : Base() {
    private val executor = Executors.newSingleThreadExecutor()
    private val device: BluetoothDevice = bleAdapter.getRemoteDevice(address)
    private val manager = PeripheralManager(device)

    val isConnected get() = manager.connected

    fun connect(): CompletableFuture<Boolean> {
        val future = CompletableFuture<Boolean>()
        executor.submit {
            if (manager.connected) {
                future.complete(true)
                return@submit
            }
            val request = manager.connect()
            request.await()
            future.complete(true)
            return@submit
        }
        return future
    }

    fun disconnect(): CompletableFuture<Boolean> {
        val future = CompletableFuture<Boolean>()
        executor.submit {
            if (!manager.connected) {
                future.complete(true)
                return@submit
            }
            val request = manager.disconnect()
            request.await()
            future.complete(true)
            return@submit
        }
        return future
    }

    fun discoverServices(): CompletableFuture<List<Service>> {
        val future = CompletableFuture<List<Service>>()
        executor.submit {
            if (manager.discovered) {
                future.complete(manager.getServices())
                return@submit
            }
            val request = manager.discoverServices()
            request.await()
            future.complete(manager.getServices())
            return@submit
        }
        return future
    }

    fun read(service: String, characteristic: String): CompletableFuture<ByteArray> {
        val future = CompletableFuture<ByteArray>()
        executor.submit {
            try {
                val serviceUuid = UUID.fromString(service)
                val characteristicUuid = UUID.fromString(characteristic)
                val request = manager.read(serviceUuid, characteristicUuid)
                request.await()
                if (request.value != null) {
                    future.complete(request.value!!)
                }
            } catch (e: IllegalArgumentException) {
                future.completeExceptionally(e)
            }
        }
        return future
    }

    fun write(service: String, characteristic: String, value: ByteArray, withoutResponse: Boolean): CompletableFuture<Boolean> {
        val future = CompletableFuture<Boolean>()
        executor.submit {
            try {
                val serviceUuid = UUID.fromString(service)
                val characteristicUuid = UUID.fromString(characteristic)
                val request = manager.write(serviceUuid, characteristicUuid, value, withoutResponse)
                request.await()
                future.complete(true)
            } catch (e: IllegalArgumentException) {
                future.completeExceptionally(e)
            }
        }
        return future
    }

    fun write(service: String, characteristic: String, value: ByteArray): CompletableFuture<Boolean> {
        return write(service, characteristic, value, false)
    }

    fun setCharacteristicNotification(service: String, characteristic: String, enable: Boolean): CompletableFuture<Boolean> {
        val future = CompletableFuture<Boolean>()
        executor.submit {
            try {
                val serviceUuid = UUID.fromString(service)
                val characteristicUuid = UUID.fromString(characteristic)
                val request = manager.setCharacteristicNotification(serviceUuid, characteristicUuid, enable)
                request.await()
                future.complete(true)
            } catch (e: IllegalArgumentException) {
                future.completeExceptionally(e)
            }
        }
        return future
    }
}
