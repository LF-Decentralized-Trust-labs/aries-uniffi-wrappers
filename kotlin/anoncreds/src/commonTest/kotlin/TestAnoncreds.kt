package org.hyperledger.anoncreds_uniffi

import anoncreds_uniffi.CredentialRevocationConfig
import anoncreds_uniffi.Disposable
import anoncreds_uniffi.Issuer
import anoncreds_uniffi.PresentationRequest
import anoncreds_uniffi.Prover
import anoncreds_uniffi.RequestedCredential
import anoncreds_uniffi.Schema
import anoncreds_uniffi.Verifier
import anoncreds_uniffi.createLinkSecret
import kotlinx.serialization.json.*
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertTrue

class TestAnoncreds {
    private lateinit var verifier: Verifier
    private lateinit var issuer: Issuer
    private lateinit var prover: Prover
    private lateinit var ffiObjects: MutableList<Disposable>

    @BeforeTest
    fun beforeEach(){
        verifier = Verifier()
        issuer = Issuer()
        prover = Prover()

        ffiObjects = mutableListOf(verifier, issuer, prover)
    }

    @AfterTest
    fun afterEach(){
        ffiObjects.forEach {
            it.destroy()
        }
    }

    @Test
    fun testAnoncredsNonce() {
        println("TESTING NONCE GENERATION")

        var low: Int? = null
        var high: Int? = null
        for(i in 0 .. 100){
            val nonce = verifier.generateNonce()
            val len = nonce.length
            if(low === null || high === null){
                low = len
                high = len
            }else{
                if(len < low)
                    low = len
                if(len > high)
                    high = len
            }
        }
        println("\tLOW NONCE LENGTH: $low")
        println("\tHIGH NONCE LENGTH: $high")
    }


    @Test
    fun createAndVerifyPresentation() {
        println("CREATE AND VERIFY PRESENTATION")

        val nonce = verifier.generateNonce()
        println("\tGENERATE NONCE: $nonce")

        val prJson = buildJsonObject {
            put("nonce", nonce)
            put("name", "pres_req_1")
            put("version", "0.1")
            putJsonObject("requested_attributes"){
                putJsonObject("attr1_referent"){
                    put("name", "name")
                    put("issuer", "mock:uri")
                }
                putJsonObject("attr2_referent"){
                    put("name", "sex")
                }
                putJsonObject("attr3_referent"){
                    put("name", "phone")
                }
                putJsonObject("attr4_referent"){
                    putJsonArray("names"){
                        add("name")
                        add("height")
                    }
                }
            }
            putJsonObject("requested_predicates"){
                putJsonObject("predicate1_referent"){
                    put("name", "age")
                    put("p_type", ">=")
                    put("p_value", 18)
                }
            }
            putJsonObject("non_revoked"){
                put("from", 10)
                put("to", 200)
            }
        }

        println("\tBUILDING PRESENTATION REQUEST FROM JSON")
        val presentationRequest = PresentationRequest(prJson.toString())
        println("\t\tPRESENTATION REQUEST: ${presentationRequest.toJson()}")
        ffiObjects.add(presentationRequest)

        println("\tBUILDING SCHEMA FROM JSON")
        val schema = Schema(buildJsonObject {
            put("name", "schema-1")
            put("issuerId", "mock:uri")
            put("version", "1")
            putJsonArray("attrNames"){
                add("name")
                add("age")
                add("sex")
                add("height")
            }
        }.toString())
        println("\t\tSCHEMA ID: ${schema.schemaId()}")
        ffiObjects.add(schema)

        println("\tCREATING CREDENTIAL DEFINITION")
        val credDefData = issuer.createCredentialDefinition(
            schema.schemaId(),
            schema,
            "TAG",
            "mock:uri",
            true
        )

        val credentialDefinition = credDefData.credDef
        val keyCorrectnessProof = credDefData.keyCorrectnessProof
        val credentialDefinitionPrivate = credDefData.credDefPriv
        ffiObjects.add(credentialDefinition)
        ffiObjects.add(keyCorrectnessProof)
        ffiObjects.add(credentialDefinitionPrivate)
        println("\t\tCREDENTIAL DEFINITION SCHEMA ID: ${credentialDefinition.schemaId()}")
        println("\t\tKEY CORRECTNESS PROOF: ${keyCorrectnessProof.toJson()}")
        println("\t\tCREDENTIAL DEFINITION PRIVATE: ${credentialDefinitionPrivate.toJson()}")
        println("\tCREATING REVOCATION REGISTRY DEFINITION")
        val revRegData = issuer.createRevocationRegistryDef(
            credentialDefinition,
            "mock:uri",
            "some_tag",
            10u,
            null
        )

        val revocationRegistryDefinition = revRegData.revRegDef
        val revocationRegistryDefinitionPrivate = revRegData.revRegDefPriv
        ffiObjects.add(revocationRegistryDefinition)
        ffiObjects.add(revocationRegistryDefinitionPrivate)
        println("\t\tREVOCATION REGISTRY DEFINITION: ${revocationRegistryDefinition.toJson()}")
        println("\t\tREVOCATION REGISTRY DEFINITION PRIVATE: ${revocationRegistryDefinitionPrivate.toJson()}")

        println("\tGETTING TAILS LOCATION")
        val tailsPath = revocationRegistryDefinition.tailsLocation()
        println("\t\tTAILS LOCATION: $tailsPath")

        println("\tCREATING REVOCATION STATUS LIST")
        val timeCreateRevStatusList = 12UL
        val revocationStatusList = issuer.createRevocationStatusList(
            credentialDefinition,
            revocationRegistryDefinition.revRegId(),
            revocationRegistryDefinition,
            revocationRegistryDefinitionPrivate,
            timeCreateRevStatusList,
            true,
        )
        ffiObjects.add(revocationStatusList)
        println("\t\tREVOCATION STATUS LIST: ${revocationStatusList.toJson()}")

        println("\tCREATING CREDENTIAL OFFER")
        val credentialOffer = issuer.createCredentialOffer(
            schema.schemaId(),
            credentialDefinition.credDefId(),
            keyCorrectnessProof
        )
        ffiObjects.add(credentialOffer)
        println("\t\tCREDENTIAL OFFER: ${credentialOffer.toJson()}")

        println("\tCREATING LINK SECRET")
        val linkSecret = createLinkSecret()
        val linkSecretId = "link secret id"
        println("\t\tLINK SECRET: $linkSecret")

        println("\tCREATING CREDENTIAL REQUEST")
        val credReqData = prover.createCredentialRequest(
            "entropy",
            null,
            credentialDefinition,
            linkSecret,
            linkSecretId,
            credentialOffer
        )

        val credentialRequest = credReqData.request
        val credentialRequestMetadata = credReqData.metadata
        ffiObjects.add(credentialRequest)
        ffiObjects.add(credentialRequestMetadata)
        println("\t\tCREDENTIAL REQUEST: ${credentialRequest.toJson()}")
        println("\t\tCREDENTIAL REQUEST METADATA: ${credentialRequestMetadata.toJson()}")

        println("\tCREATING CREDENTIAL")
        val credentialRevocationConfig = CredentialRevocationConfig(
            revocationRegistryDefinition,
            revocationRegistryDefinitionPrivate,
            revocationStatusList,
            9U
        )
        var credential = issuer.createCredential(
            credentialDefinition,
            credentialDefinitionPrivate,
            credentialOffer,
            credentialRequest,
            mapOf(
                "name" to "Alex",
                "height" to "175",
                "age" to "28",
                "sex" to "male"
            ),
            null,
            credentialRevocationConfig
        )
        ffiObjects.add(credential)
        ffiObjects.add(credentialRevocationConfig)
        println("\t\tCREDENTIAL: ${credential.toJson()}")

        println("\tPROCESSING CREDENTIAL")
        credential = prover.processCredential(
            credential,
            credentialRequestMetadata,
            linkSecret,
            credentialDefinition,
            revocationRegistryDefinition
        )
        ffiObjects.add(credential)
        println("\t\tPROCESSED CREDENTIAL: ${credential.toJson()}")

        println("\tGETTING REVOCATION REGISTRY INDEX")
        val revocationRegistryIndex = credential.revRegIndex()
        println("\t\tREVOCATION REGISTRY INDEX: $revocationRegistryIndex")

        println("\tCREATING REVOCATION STATE")
        val revocationState = prover.createOrUpdateRevocationState(
            revocationRegistryDefinition,
            revocationStatusList,
            revocationRegistryIndex!!,
            tailsPath,
            null,
            null
        )
        ffiObjects.add(revocationState)
        println("\t\tREVOCATION STATE: ${revocationState.toJson()}")

        println("\tCREATING PRESENTATION")

        val present = RequestedCredential(
            credential,
            timeCreateRevStatusList,
            revocationState,
            mapOf(
                "attr1_referent" to true,
                "attr2_referent" to false,
                "attr4_referent" to true,
            ),
            listOf("predicate1_referent")
        )
        ffiObjects.add(present)
        val presentation = prover.createPresentation(
            presentationRequest,
            listOf(present),
            mapOf("attr3_referent" to "8-800-300"),
            linkSecret,
            mapOf(schema.schemaId() to schema),
            mapOf(credentialDefinition.credDefId() to credentialDefinition)
        )
        ffiObjects.add(presentation)
        println("\t\tPRESENTATION: ${presentation.toJson()}")

        println("\tVERIFYING PRESENTATION")
        val verify = verifier.verifyPresentation(
            presentation,
            presentationRequest,
            mapOf(schema.schemaId() to schema),
            mapOf(credentialDefinition.credDefId() to credentialDefinition),
            mapOf(revocationRegistryDefinition.revRegId() to revocationRegistryDefinition),
            listOf(revocationStatusList),
            null
        )

        println("\t\tVERIFICATION RESULT: $verify")

        assertTrue(verify)
    }

}