package com.nonpolynomial.btleplug.droidplug

import android.annotation.TargetApi
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanSettings
import android.os.Build
import android.os.ParcelUuid
import org.astonbitecode.j4rs.api.Instance
import org.astonbitecode.j4rs.api.java2rust.Java2RustUtils
import android.bluetooth.le.ScanFilter as BleScanFilter
import android.bluetooth.le.ScanResult as BleScanResult


class AdapterException(message: String): RuntimeException(message) {
}


@Suppress("unused")
// Native code uses this class.
class Adapter: Base() {

    private val scanCallback = object: ScanCallback() {
        override fun onScanResult(callbackType: Int, result: BleScanResult?) {
            if (result != null) {
                val address = result.device.address
                val addressType = null
                val scanRecord = result.scanRecord
                var localName: String?
                try {
                    localName = result.device.name
                } catch (e: SecurityException) {
                    localName = null
                }
                var txPowerLevel: Int? = null
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    txPowerLevel = result.txPower
                }
                val rssi = result.rssi
                val manufacturerData = HashMap<Int, ByteArray>()
                for (i in 0 until (scanRecord?.manufacturerSpecificData?.size()?:0)) {
                    val key = scanRecord!!.manufacturerSpecificData!!.keyAt(i)
                    val data = scanRecord.manufacturerSpecificData?.get(i)
                    if (data != null) {
                        manufacturerData[key] = data
                    }
                }
                val serviceData = HashMap<String, ByteArray>()
                scanRecord?.serviceData?.forEach { serviceData[it.key.toString()] = it.value }
                val scanResult = ScanResult(
                        address,
                        addressType,
                        localName,
                        txPowerLevel,
                        rssi,
                        manufacturerData,
                        serviceData,
                        scanRecord?.serviceUuids?.map { it.toString() } ?: emptyList<String>()
                )
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
            bleScanner.startScan(filters, settings, scanCallback)
        } catch (ex: SecurityException) {
            throw AdapterException("Missing bluetooth scan permission")
        }
    }

    fun stopScan() {
        try {
            bleScanner.stopScan(scanCallback)
        } catch (ex: SecurityException) {
            throw AdapterException("Missing bluetooth scan permission")
        }
    }

    private external fun reportScanResult(scanResult: Instance<ScanResult>)
}
