package com.nonpolynomial.btleplug.droidplug

import android.annotation.TargetApi
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.BluetoothLeScanner
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.Build
import android.os.ParcelUuid
import org.astonbitecode.j4rs.api.Instance
import org.astonbitecode.j4rs.api.java2rust.Java2RustUtils
import org.jetbrains.annotations.NotNull
import android.bluetooth.le.ScanFilter as BleScanFilter
import android.bluetooth.le.ScanResult as BleScanResult


class AdapterException(message: String): RuntimeException(message) {
}


@Suppress("unused")
// Native code uses this class.
class Adapter () {
    companion object {
        private var bleAdapter: BluetoothAdapter? = null;

        fun initialize(@NotNull context: Context) {
            bleAdapter = (context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager).adapter
        }
    }


    private val scanCallback = object: ScanCallback() {
        override fun onScanResult(callbackType: Int, result: BleScanResult?) {
            if (result != null) {
                val scanResult = ScanResult()
                scanResult.address = result.device.address
                scanResult.addressType = null
                val scanRecord = result.scanRecord
                try {
                    scanResult.localName = result.device.name
                } catch (e: SecurityException) {
                    scanResult.localName = null
                }
                @TargetApi(Build.VERSION_CODES.O)
                scanResult.txPowerLevel = result.txPower
                scanResult.rssi = result.rssi
                for (i in 0 until (scanRecord?.manufacturerSpecificData?.size()?:0)) {
                    val key = scanRecord!!.manufacturerSpecificData!!.keyAt(i)
                    val data = scanRecord.manufacturerSpecificData?.get(i)
                    if (data != null) {
                        scanResult.manufacturerData[key] = data
                    }
                }
                scanRecord?.serviceData?.forEach { scanResult.serviceData[it.key.toString()] = it.value }
                scanRecord?.serviceUuids?.forEach { scanResult.services.add(it.toString()) }
                this@Adapter.reportScanResult(Java2RustUtils.createInstance(scanResult))
            }
        }
    }

    fun startScan(filter: ScanFilter) {
        var filters: List<BleScanFilter>? = null
        val uuids = filter.getUuids()
        if (uuids.isNotEmpty()) {
            filters = uuids.map { BleScanFilter.Builder().setServiceUuid(ParcelUuid.fromString(it.toString())).build() }
        }
        val settings = ScanSettings.Builder().setCallbackType(ScanSettings.CALLBACK_TYPE_ALL_MATCHES).build()
        try {
            getLeScanner().startScan(filters, settings, scanCallback)
        } catch (ex: SecurityException) {
            throw AdapterException("Missing bluetooth scan permission")
        }
    }

    fun stopScan() {
        try {
            getLeScanner().stopScan(scanCallback)
        } catch (ex: SecurityException) {
            throw AdapterException("Missing bluetooth scan permission")
        }
    }

    private fun getLeScanner(): BluetoothLeScanner {
        if (bleAdapter == null) {
            throw AdapterException("Bluetooth unavailable")
        }
        return bleAdapter!!.bluetoothLeScanner;
    }

    private external fun reportScanResult(scanResult: Instance<ScanResult>)
}
