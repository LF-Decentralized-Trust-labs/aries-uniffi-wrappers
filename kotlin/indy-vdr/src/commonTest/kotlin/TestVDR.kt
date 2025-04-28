package org.hyperledger.indy_vdr_uniffi

import indy_vdr_uniffi.Disposable
import indy_vdr_uniffi.Ledger
import indy_vdr_uniffi.LedgerType
import indy_vdr_uniffi.openPool
import indy_vdr_uniffi.use
import kotlinx.serialization.json.*
import kotlin.test.Test
import kotlinx.coroutines.runBlocking
import kotlinx.datetime.Clock
import org.hyperledger.indy_vdr_uniffi.GenesisFile
import kotlin.test.AfterTest
import kotlin.test.BeforeTest

class TestVDR {

    private val ffiObjects:MutableList<Disposable> = mutableListOf()

    @AfterTest
    fun afterEach(){
        ffiObjects.forEach { disposable ->
            disposable.destroy()
        }
    }

    @Test
    fun indyVDR(){
        runBlocking{
            println("Opening pool...")
            val pool = openPool(null, GenesisFile, null)
            ffiObjects.add(pool)

            println("\tPool Status: ${pool.getStatus()}")

            pool.refresh()
            println("\tPool Status after refresh: ${pool.getStatus()}")

            val ledger = Ledger()
            ffiObjects.add(ledger)

            println("Creating request body...")
            val testReq = """{
                "operation": {"data": 1, "ledgerId": 1, "type": "3"},
                "protocolVersion": 2,
                "reqId": 123,
                "identifier": "LibindyDid111111111111"
            }"""
            var req = ledger.buildCustomRequest(testReq)
            ffiObjects.add(req)
            println("\tRequest body: ${req.body()}")
            println("\tRequest signature input: ${req.signatureInput()}")

            println("Submitting get txn agreement request...")
            req = ledger.buildGetTxnAuthorAgreementRequest(null, null)
            ffiObjects.add(req)

            val poolResponse = pool.submitRequest(req)
            println("\tPool response: $poolResponse")

            println("Submitting get acceptance mechanism request...")
            req = ledger.buildGetAcceptanceMechanismsRequest(null, null, null)
            ffiObjects.add(req)
            println("\tPool response: ${pool.submitRequest(req)}")


            println("Accepting TAA")
            val acceptance = ledger.prepareTxnAuthorAgreementAcceptance(
                "acceptance text",
                "1.1.1",
                null,
                "manual",
                Clock.System.now().epochSeconds.toULong()
            )
            req = ledger.buildGetTxnRequest(null, LedgerType.DOMAIN, 15)
            req.setTxnAuthorAgreementAcceptance(acceptance)
            req.setEndorser("V4SGRU86Z58d6TV7PBUe6f")
            req.setMultiSignature("V4SGRU86Z58d6TV7PBUe6f", "sig".encodeToByteArray())
            println("\tTAA acceptance request: ${req.use{it.body()}}")

            req = ledger.buildGetTxnRequest(null, LedgerType.DOMAIN, 1)
            ffiObjects.add(req)
            println("Pool response: ${pool.submitRequest(req)}")

            req = ledger.buildGetSchemaRequest(null, "6qnvgJtqwK44D8LFYnV5Yf:2:relationship.dflow:1.0.0")
            println("Get schema request: ${req.use{it.body()}}")

            req = ledger.buildGetCredDefRequest(null, "A9Rsuu7FNquw8Ne2Smu5Nr:3:CL:15:tag")
            println("Get cred def request: ${req.use{it.body()}}")

            val revocId = "L5wx9FUxCDpFJEdFc23jcn:4:L5wx9FUxCDpFJEdFc23jcn:3:CL:1954:default:CL_ACCUM:c024e30d-f3eb-42c5-908a-ff885667588d"

            req = ledger.buildGetRevocRegDefRequest(null, revocId)
            println("Get revoc reg def request: ${req.use{it.body()}}")

            req = ledger.buildGetRevocRegRequest(null, revocId, 1)
            println("Get revoc reg request: ${req.use{it.body()}}")

            req = ledger.buildGetRevocRegDeltaRequest(null, revocId, null, 1)
            println("Get revoc reg delta request: ${req.use{it.body()}}")

            println("Closing pool...")
            pool.closePool()
            println("\tPool closed.")
        }
    }

}