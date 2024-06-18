
string1 = "DOORDASH-ALL-THE-ADS-2024-PROMOCODE-DOORDASHCANDOORDASHPRETTYMUCHANYTHING-ANEWKIAEV9-REESES-PEANUTBUTTERM&MS-HOPEYOUHAVEADVR-FANDUEL-DINAMITA-MOUNTAINDEWBAJABLAST-BMWi5-POPEYESCHICKEN-YOUTUBETV-OREO-DOVE-LIVEDASHLOVE-HOPE-YOU’RE-NOT-MISSING-ANYTHING-DOORS-IAMNOTGOINGTOREADALLOFTHISBUTYOUGETTHEIDEA-MORE-BAJA-BLAST-ANYONE?-MAYONNAISE-LOTS-AND-LOTS-OF-MAYONNAISE-PERSONAL-WEBSITE-CREATION-TECHNOLOGY-WHERE-ARE-YOU-GOING-TO-PUT-ALL-THIS-STUFF?-YOUR-GARAGE-IS-TOO-SMALL-OH-IT’S-STILL-GOING-INTO-THE-YARD-WOW-LOOK-ANOTHER-CAR-IT'S-A-VERY-LONG-PROMO-CODE!“YOUR-DOOR-TO-MORE: A-SHORT-ESSAY-ABOUT-STUFF”-FOR-TOO-LONG-COMMERCIALS-HAVE-SIMPLY-ADVERTISED-GOODS-AND-OR-SERVICES-BUT-NEVER-HAS-COMMERCIAL-GOTTEN-YOU-GOODS-AND-OR-SERVICES-FROM-ALL-THE-OTHER-COMMERCIALS-VIA-THE-CORRECT-APPLICATION-OF-AN-EXTENSIVE-ALPHANUMERIC-PROMOTIONAL-CODE-LIKE-CHEESEBURGERS-MIDSIZE-GERMAN-AUTOMOBILES-STARRY-38230-90580AM8028HS7A8D99-ET-CETERA-YOU-GET-IT-HOWEVER-THE-DESIRED-OUTCOME-RESULTS-IN-AN-INFLUX-OF-MULTIPLE-AUTOMOBILES-WHICH-CAN-BE-OVERWHELMING-FOR-THE-SENSES-BECAUSE-THE-NEW-CAR-SMELL-OF-ONE-CAR-IS-QUITE-INCREDIBLE-BUT-THE-NEW-CAR-SMELL-OF-MULTIPLE-NEW-CARS-HAS-THE-POTENTIAL-TO-CREATE-A-SENSE-OF-EUPHORIA-UNATTAINABLE-ANYWHERE-ELSE-IN-THE-NATURAL-WORLD-THE-SAME-IS-TRUE-WHEN-ONE-OBTAINS-THE-SNACKS-FROM-ALL-THE-ADS-AS-WE-HUMANS-OFTEN-FEEL-DECISION-PARALYSIS-GIVEN-SO-MANY-DELICIOUS-OPTIONS-AT-THE-SAME-TIME-AND-THEN-HAVING-ACCESS-TO-TURBOTAX-EXPERTS-MAKES-THINGS-SO-INCREDIBLY-CONVENIENT-THAT-ALL-OTHER-ASPECTS-OF-LIFE-BECOME-LESS-CONVENIENT-BY-COMPARISON-THIS-IS-BOTH-A-CONUNDRUM-AND-THE-BEST-THING-TO-EVER-HAPPEN-TO-YOU-BUT-MOSTLY-THE-BEST-THING-TO-EVER-HAPPEN-TO-YOU-AND-SORRY-ABOUT-ALL-THE-DASHES-JUST-KIDDING-HERE-ARE-LIKE-FIFTEEN-OR-SO-MORE-BUT-YOU-WILL-NEED-TO-COUNT-THEM-TO-SEE-IF-THAT-IS-CORRECT-DASH-DASH-DASH-!!!!!"
string2 = """DOORDASH-ALL-THE-ADS-2024-PROMOCODE-DOORDASHCANDOORDASHPRETTYMUCHANYTHING-ANEWKIAEV9-REESES-PEANUTBUTTERM&MS-HOPEYOUHAVEADVR-FANDUEL-DINAMITA-MOUNTAINDEWBAJABLAST-BMWi5-POPEYESCHICKEN-YOUTUBETV-OREO-DOVE-LIVEDASHLOVE-HOPE-YOU'RE-NOT-MISSING-ANYTHING-DOORS-IAMNOTGOINGTOREADALLOFTHISBUTYOUGETTHEIDEA-MORE-BAJA-BLAST-ANYONE?-MAYONNAISE-LOTS-AND-LOTS-OF-MAYONNAISE-PERSONAL-WEBSITE-CREATION-TECHNOLOGY-WHERE-ARE-YOU-GOING-TO-PUT-ALL-THIS-STUFF?-YOUR-GARAGE-IS-TOO-SMALL-OH-IT'S-STILL-GOING-INTO-THE-YARD-WOW-LOOK-ANOTHER-CAR-IT'S-A-VERY-LONG-PROMO-CODE!"YOUR-DOOR-TO-MORE:A-SHORT-ESSAY-ABOUT-STUFF"-FOR-TOO-LONG-COMMERCIALS-HAVE-SIMPLY-ADVERTISED-GOODS-AND-OR-SERVICES-BUT-NEVER-HAS-A-COMMERCIAL-GOTTEN-YOU-GOODS-AND-OR-SERVICES-FROM-ALL-THE-OTHER-COMMERCIALS-VIA-THE-CORRECT-APPLICATION-OF-AN-EXTENSIVE-ALPHANUMERIC-PROMOTIONAL-CODE-LIKE-CHEESEBURGERS-MIDSIZE-GERMAN-AUTOMOBILES-STARRY-38230-90580AM8028HS7A8D99-ET-CETERA-YOU-GET-IT-HOWEVER-THE-DESIRED-OUTCOME-RESULTS-IN-AN-INFLUX-OF-MULTIPLE-AUTOMOBILES-WHICH-CAN-BE-OVERWHELMING-FOR-THE-SENSES-BECAUSE-THE-NEW-CAR-SMELL-OF-ONE-CAR-IS-QUITE-INCREDIBLE-BUT-THE-NEW-CAR-SMELL-OF-MULTIPLE-NEW-CARS-HAS-THE-POTENTIAL-TO-CREATE-A-SENSE-OF-EUPHORIA-UNATTAINABLE-ANYWHERE-ELSE-IN-THE-NATURAL-WORLD-THE-SAME-IS-TRUE-WHEN-ONE-OBTAINS-THE-SNACKS-FROM-ALL-THE-ADS-AS-WE-HUMANS-OFTEN-FEEL-DECISION-PARALYSIS-GIVEN-SO-MANY-DELICIOUS-OPTIONS-AT-THE-SAME-TIME-AND-THEN-HAVING-ACCESS-TO-TURBOTAX-EXPERTS-MAKES-THINGS-SO-INCREDIBLY-CONVENIENT-THAT-ALL-OTHER-ASPECTS-OF-LIFE-BECOME-LESS-CONVENIENT-BY-COMPARISON-THIS-IS-BOTH-A-CONUNDRUM-AND-THE-BEST-THING-TO-EVER-HAPPEN-TO-YOU-BUT-MOSTLY-THE-BEST-THING-TO-EVER-HAPPEN-TO-YOU-AND-SORRY-ABOUT-ALL-THE-DASHES-JUST-KIDDING-HERE-ARE-LIKE-FIFTEEN-OR-SO-MORE-BUT-YOU-WILL-NEED-TO-COUNT-THEM-TO-SEE-IF-THAT-IS-CORRECT-DASH-DASH-DASH-!!!!!"""

# string1 = string1.split('-')
# string2 = string2.split('-')

i = 0
k = 0
errors = 0
for i in range(len(string1)):
    if(errors < 15):
        if string1[i] == string2[k]:
            i += 1
            k += 1
        else:
            errors += 1
            print("\nInconsistency here: ")
            print("Actual: " + string2[k-5:k+5])
            print("Claire: " + string1[i-5:i+5])
            print("error:       ^")
            if string1[i+1] == string2[k+1]:
                i += 1
                k += 1

            elif string1[i+1] == string2[k]:
                i += 1
            elif string1[i] == string2[k+1]:
                k += 1
            elif string1[i+2] == string2[k+1]:
                i += 2
                k += 1
            elif string1[i+1] == string2[k+2]:
                k += 2
                i += 1
            elif string1[i+2] == string2[k+2]:
                i += 2
                k += 2
            
            else:
                if string2[k] == '-':
                    k += 1
                    while string2[k] != '-':
                        k += 1
                    while string1[i] != '-':
                        i += 1
                elif string1[i] == '-':
                    i += 1
                    while string1[i] != '-':
                        i += 1
                    while string2[k] != '-':
                        k += 1
                else:
                    while string2[k] != string1[i]:
                        k += 1
                        #print("While loop " + string2[k] + " " + string1[i])
                    i += 1
                    k += 1
                    
            
            # else:
            #     if string2[k] == '-':
            #         k += 1
            #     while string2[k] != '-':
            #         k += 1
            #     k += 1
            #     nextTag = string2[k:k+1]
            #     while(string1[i:i+1] != nextTag):
            #         i += 1

            # print("testi: " + string1[i+1])
            # print("testk: " + string2[k])
            
            # if string1[i+1] == string2[k+1]:
            #     i += 1
            #     k += 1
            # elif string1[i+1] == string2[k]:
            #     i += 1
            # elif string1[i] == string2[k+1]:
            #     k += 1
            # elif string1[i+2] == string2[k+1]:
            #     i += 2
            #     k += 1
            # elif string1[i+1] == string2[k+2]:
            #     k += 2
            #     i += 1
            
            # else:
            #     k += 2
            #     while string1[i] != string2[k]:
            #         i += 1
            #     # print("Could not find next correct")
            #     # exit()

    else:
        print("\nerror limit reached")
        exit()

print("\n Finished!")    