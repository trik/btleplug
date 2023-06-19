package com.nonpolynomial.btleplug.droidplug

import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.os.ConditionVariable
import android.util.Log

sealed class Request(private val manager: PeripheralManager) {
    var finished = false
    private val syncLock = ConditionVariable(true)

    private var failedCallback: RequestCallback? = null
    private var completedCallback: RequestCallback? = null

    fun notify(status: Int) {
        if (status == BluetoothGatt.GATT_SUCCESS) {
            notifyCompleted()
        } else {
            notifyFailed(status)
        }
    }

    fun notifyCompleted() {
        if (!finished) {
            finished = true;
        }
        if (completedCallback == null) {
            Log.d("iddio", "no completed callback")
        }
        completedCallback?.onCompleted()
    }

    fun notifyFailed(status: Int) {
        if (!finished) {
            finished = true;
        }
        if (failedCallback == null) {
            Log.d("iddio", "no failed callback")
        }
        failedCallback?.onFailed(status)
    }

    fun await() {
        try {
            if (finished) {
                throw IllegalStateException()
            }
            syncLock.close()
            val callback = RequestCallback()
            failedCallback = callback
            completedCallback = callback
            manager.enqueue(this)
            syncLock.block()
            if (!callback.successful) {
                // raise some exception
            }
        } finally {

        }
    }

    override fun toString(): String {
        return "Request"
    }

    inner class RequestCallback {
        var status = BluetoothGatt.GATT_SUCCESS;

        val successful get() = status == BluetoothGatt.GATT_SUCCESS

        private val reqSyncLock = syncLock

        fun onCompleted() {
            Log.d("iddio", "onCompleted")
            reqSyncLock.open();
        }

        fun onFailed(status: Int) {
            this.status = status
            Log.d("iddio", "onFailed")
            reqSyncLock.open()
        }
    }

    companion object {
        const val REASON_REQUEST_INVALID = -1000000
        const val REASON_MISSING_PERMISSIONS = -1000001
        const val REASON_INVALID_REQUEST = -1000002
        const val REASON_INVALID_CHARACTERISTIC = -1000003
        const val REASON_WRITE_FAILED = -1000004
        const val REASON_SET_NOTIFICATION_FAILED = -1000005
    }
}
