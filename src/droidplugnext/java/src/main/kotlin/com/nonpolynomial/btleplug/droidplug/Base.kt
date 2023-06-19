package com.nonpolynomial.btleplug.droidplug

import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.BluetoothLeScanner
import android.content.Context

open class Base {
    companion object {
        lateinit var contextProvider: AdapterContextProvider
            private set
        val bleAdapter get(): BluetoothAdapter = (contextProvider.context!!.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager).adapter
        val bleScanner get(): BluetoothLeScanner = bleAdapter.bluetoothLeScanner

        fun initialize(provider: AdapterContextProvider) {
            contextProvider = provider
        }
    }
}