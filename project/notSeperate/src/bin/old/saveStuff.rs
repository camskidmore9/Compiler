            // tokenTypeEnum::L_PAREN => {
            //     let mut k = i + 1; // Start from the token right after '('
            //     // println!("\nFound a (");
            //     let mut curStmt: Vec<&Token> = vec![];
            //     let mut depth = 1; // Track nested parentheses depth
            
            //     while k < tokenList.len() {
            //         let nextTok = &tokenList[k];
            //         // println!("Current token: {}", nextTok.tokenString);
            
            //         if nextTok.tt == tokenTypeEnum::L_PAREN {
            //             // println!("Sub statement found");
            //             depth += 1;
            //         } else if nextTok.tt == tokenTypeEnum::R_PAREN {
            //             // println!("Closing bracket found");
            //             depth -= 1;
            
            //             if depth == 0 {
            //                 // End of the nested parentheses block
            //                 curStmt.push(nextTok);
            //                 break;
            //             }
            //         }
            
            //         curStmt.push(nextTok);
            //         k += 1;
            //     }

            //     let newTokList: Vec<Token> = curStmt.iter().cloned().map(|t| t.clone()).collect();
            //     let scanned = self.parse(newTokList, 0);
            
            //     match scanned {
            //         Ok((reporting, Some(stmt))) => {
            //             // println!("Parsed nested statement: {:?}", stmt);
            //             // Push the parsed statement into newBlock
            //             let result = self.processBlock(&stmt);

            //             if result.is_ok() {
            //                 let expr = result.unwrap();
            //                 // println!("Extracted Expr: {:?}", expr);

            //                 let exprStmt = Stmt::Expr(expr);

            //                 // println!("testStmt: {}", testStmt);
            //                 let _ = newBlock.push_to_block(exprStmt);

            //             } else {
            //                 println!("Failed to extract Expr in l_paren: {}", result.unwrap_err());
            //             }

                        
            //         },
            //         Ok((reporting, None)) => {
            //             println!("Parsed nested statement but no statement returned.");
            //             // Handle the case where no statement is returned (if needed)
            //         },
            //         Err(reporting) => {
            //             println!("Error parsing nested statement: {:?}", reporting);
            //             return Err(reporting); // Propagate the error up the call stack
            //         },
            //     }
            
            //     i = k + 1; // Move index past the ')' token
            // }
            // tokenTypeEnum::PROCEDURE => {
            //     //Finds the end of the procedure statement
            //     let mut k = i + 1;
            //     let mut nextTok = &tokenList[k];
            //     println!("\n\nFound a procedure");
            //     let mut curStmt: Vec<&Token> = vec![];
            
            //     // Finds the end of the if
            //     curStmt.push(token);
            //     while nextTok.tt != tokenTypeEnum::END_PROCEDURE {
            //         curStmt.push(nextTok);
            //         k = k + 1;
            //         nextTok = &tokenList[k];
            //     }
            //     curStmt.push(nextTok);

            //     let procId = &curStmt[1].tokenString;
            //     let procType = VarType::new(&curStmt[3].tokenString);

            //     println!("Found the end of a procedure");

            //     //Gets the procedure type
            //     match procType {
            //         Ok(varType) => {
            //             println!("Procedure type: {:?}", varType);
            //             println!("Procedure id: {}", procId);

            //         }
            //         Err(err) => println!("Error determining procedure type: {}", err),
            //     }

            //     let mut paramList = Stmt::Block(Vec::new());
                
            //     let mut j = 5;
            //     //Finds the end of the parameters
            //     if(curStmt[4].tt != tokenTypeEnum::L_PAREN){
            //         println!("Not parentheses: {}", &curStmt[4].tt);
            //     } else {
            //         //Finds the end of the procedure statement
            //         let mut nextTok = &curStmt[j];
            //         // println!("\n\nFound a procedure");
            //         let mut paramTokens: Vec<&Token> = vec![];
            //         let decLine = curStmt[4].lineNum.clone();
            //         // Finds the end of the if
            //         // curStmt.push(token);
            //         while nextTok.tt != tokenTypeEnum::R_PAREN  {
            //             if(nextTok.lineNum != decLine){
            //                 println!("No right parent, make error");
            //             } else {
            //                 paramTokens.push(nextTok);
            //                 j = j + 1;
            //                 nextTok = &curStmt[j];
            //             }
            //         }

            //         // println!("Found all parameters:");
            //         // for token in &paramTokens {
            //         //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
            //         // }



            //         let mut curParam: Vec<&Token> = vec![];
            //         for token in &paramTokens {
            //             if(token.tt == tokenTypeEnum::COMMA) {
            //                 //Parse the parameter
            //                 let tokenString: String = ";".to_string();
            //                 let semicolon = Token::new(crate::tokenTypeEnum::SEMICOLON,tokenString, decLine.to_string());
            //                 curParam.push(&semicolon);
            //                 let newCurParam: Vec<Token> = curParam.iter().cloned().map(|t| t.clone()).collect();
            //                 let scanParam = self.parse(newCurParam, 0);
            //                 let mut paramBlock: Option<Stmt>;
            //                 match scanParam {
            //                     Ok((reporting, Some(stmt))) => {
            //                         // Add your logic to handle the parsed condition statement here
            //                         // For example:
            //                         // println!("Good if: {:?}", stmt);
            //                         paramBlock = Some(stmt); // Assuming Stmt is the type of your condition
            //                         // Add condition to your newBlock or handle it as needed
            //                     },
            //                     Ok((reporting, None)) => {
            //                         println!("Parsed parameter but no statement returned.");
            //                         paramBlock = None; // Assuming Stmt is the type of your condition

            //                         self.reports.reportError(format!(
            //                             "In line: {}, Error with parameter", curStmt[0].lineNum
            //                         ));

            //                     },
            //                     Err(reporting) => {
            //                         println!("Error parsing condition: {:?}", reporting);
            //                         println!("Parsed condition but no statement returned.");
            //                         paramBlock = None; // Assuming Stmt is the type of your condition
            //                         self.reports.reportError(format!(
            //                             "In line: {}, Error with condition", curStmt[0].lineNum
            //                         ));
            //                     },
            //                 }
            //                 if let Some(param) = paramBlock {
                                    
            //                     let result = self.processBlockStmt(&param);

            //                     if result.is_ok() {
            //                         let param = result.unwrap();

            //                         // let paramStmt = Stmt::If(expr, Box::new(ifCond), None);
            //                         // println!("Here is the if parameter: {:?}", param);
            //                         let _ = paramList.push_to_block(param);
            //                         // let _ = newBlock.push_to_block(ifStmt);


            //                     } else {
            //                         println!("Failed to extract Expr in param: {}", result.unwrap_err());
            //                     }
                                    
                                    
            //                 } else {
            //                     println!("error in if statment, need to write");
            //                 }
            //                 curParam = vec![];
            //             } else {
            //                 curParam.push(token);
            //             }
            //         }
            //         if((paramTokens.len().clone() as i32) != 0){
            //             let tokenString: String = ";".to_string();
            //             let semicolon = Token::new(crate::tokenTypeEnum::SEMICOLON,tokenString, decLine.to_string());
            //             curParam.push(&semicolon);
            //             let newCurParam: Vec<Token> = curParam.iter().cloned().map(|t| t.clone()).collect();
            //             let scanParam = self.parse(newCurParam, 0);
            //             let mut paramBlock: Option<Stmt>;
            //             match scanParam {
            //                 Ok((reporting, Some(stmt))) => {
            //                     // Add your logic to handle the parsed condition statement here
            //                     // For example:
            //                     // println!("Good if: {:?}", stmt);
            //                     paramBlock = Some(stmt); // Assuming Stmt is the type of your condition
            //                     // Add condition to your newBlock or handle it as needed
            //                 },
            //                 Ok((reporting, None)) => {
            //                     println!("Parsed parameter but no statement returned.");
            //                     paramBlock = None; // Assuming Stmt is the type of your condition

            //                     self.reports.reportError(format!(
            //                         "In line: {}, Error with parameter", curStmt[0].lineNum
            //                     ));

            //                 },
            //                 Err(reporting) => {
            //                     println!("Error parsing condition: {:?}", reporting);
            //                     println!("Parsed condition but no statement returned.");
            //                     paramBlock = None; // Assuming Stmt is the type of your condition
            //                     self.reports.reportError(format!(
            //                         "In line: {}, Error with condition", curStmt[0].lineNum
            //                     ));
            //                 },
            //             }
            //             if let Some(param) = paramBlock {
                                
            //                 let result = self.processBlockStmt(&param);

            //                 if result.is_ok() {
            //                     let param = result.unwrap();

            //                     // let paramStmt = Stmt::If(expr, Box::new(ifCond), None);
            //                     // println!("Here is the if parameter: {:?}", param);
            //                     let _ = paramList.push_to_block(param);
            //                     // let _ = newBlock.push_to_block(ifStmt);


            //                 } else {
            //                     println!("Failed to extract Expr in param: {}", result.unwrap_err());
            //                 }
                                
                                
            //             } else {
            //                 println!("error in if statment, need to write");
            //             }
            //         }
            //     }


            //     // println!("Procedure tokens: ");
            //     // for token in &curStmt {
            //     //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
            //     // }

            //     println!("Params: ");
            //     paramList.display(0);

            //     println!("Next token: {}", &curStmt[j+2].tokenString);

            //     curStmt.drain(0..j+1);

            //     // println!("remaining Procedure tokens: ");
            //     // for token in &curStmt {
            //     //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
            //     // }

            //     let newCurParam: Vec<Token> = curStmt.iter().cloned().map(|t| t.clone()).collect();
                
            //     // println!("new curStmt: ");
            //     // for token in &newCurParam {
            //     //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
            //     // }
                
                
            //     let scanParam = self.parse(newCurParam, 0);
            //     let mut paramBlock: Option<Stmt>;
            //     match scanParam {
            //         Ok((reporting, Some(stmt))) => {
            //             // Add your logic to handle the parsed condition statement here
            //             // For example:
            //             // println!("Good if: {:?}", stmt);
            //             paramBlock = Some(stmt); // Assuming Stmt is the type of your condition
            //             // Add condition to your newBlock or handle it as needed
            //         },
            //         Ok((reporting, None)) => {
            //             println!("Parsed procedure but no statement returned.");
            //             paramBlock = None; // Assuming Stmt is the type of your condition

            //             self.reports.reportError(format!(
            //                 "In line: {}, Error with procedure", curStmt[0].lineNum
            //             ));

            //         },
            //         Err(reporting) => {
            //             println!("Error parsing procedure: {:?}", reporting);
            //             println!("Parsed procedure but no statement returned.");
            //             paramBlock = None; // Assuming Stmt is the type of your condition
            //             self.reports.reportError(format!(
            //                 "In line: {}, Error with procedure", curStmt[0].lineNum
            //             ));
            //         },
            //     }
            //     if let Some(param) = paramBlock {
                        
            //         let result = self.processBlockStmt(&param);

            //         if result.is_ok() {
            //             let param = result.unwrap();

            //             // let paramStmt = Stmt::If(expr, Box::new(ifCond), None);
            //             println!("Here is the procedure: {:?}", param);
            //             // let _ = paramList.push_to_block(param);
            //             // let _ = newBlock.push_to_block(ifStmt);


            //         } else {
            //             println!("Failed to extract Expr in procedure: {}", result.unwrap_err());
            //         }
                        
                        
            //     } else {
            //         println!("error in procedure, need to write");
            //     }




            

            //     // println!("K: {}", tokenList[k].tokenString);
            //     i = k + 1; // Move to the next token after the END_IF
            // }








            // tokenTypeEnum::BEGIN => {
            //     let mut retStmt:Stmt;
            //     let mut k = 0;
            //     let mut nextTok = &tokenList[k];
            //     println!("\nFound a program begin");
            //     let mut curStmt: Vec<Token> = vec![];
            //     curStmt.push(token.clone());
            //     while (nextTok.tt != tokenTypeEnum::END_PROGRAM) && (nextTok.tt != tokenTypeEnum::END_PROCEDURE) {
            //         curStmt.push(nextTok.clone());
            //         k = k + 1;
            //         nextTok = &tokenList[k];
            //     }
            //     curStmt.push(nextTok.clone());
            //     // println!("Found the end program");
                
            //     curStmt.remove(0);
            
            //     // for token in &curStmt {
            //     //     println!("< \"{}\" , {}, {} >", token.tokenString, token.tt.to_string(), token.lineNum);
            //     // }
            
            //     // let progBlock = ;
            //     let subLen = curStmt.len().clone();

            //     match self.parse(&mut curStmt) {
            //         Ok((reporting, Some(stmt))) => {
            //             // println!("\n\nParsing succeeded.");
            //             // println!("Reporting: {:?}", reporting);
            //             // println!("Parsed Statement: {:?}", stmt);
            //             // println!("Returned block: {}", stmt);

            //             let retStmt = stmt;

            //             // Continue with normal flow
            //         }
            //         Ok((reporting, None)) => {
            //             // println!("\n\nParsing succeeded, but no statement was returned.");
            //             // println!("Reporting: {:?}", reporting);
            //             // Continue with normal flow
            //         }
            //         Err(reporting) => {
            //             // eprintln!("\n\nParsing failed.");
            //             // eprintln!("Reporting: {:?}", reporting);
            //             // Handle the error gracefully, log, recover, etc.
            //         }
            //     }



                
            //     i = i + subLen;
            // }




            tokenTypeEnum::IDENTIFIER => {
                let mut retStmt:Stmt;
                
                let mut k = 0;
                let mut nextTok = &tokenList[k];
                // println!("Found an identifier");
                let mut curStmt: Vec<&Token> = vec![];
                // curStmt.push(token);
                while k < tokenList.len() {
                    let nextTok = &tokenList[k];
                    curStmt.push(nextTok);
                
                    if (nextTok.tt == tokenTypeEnum::SEMICOLON) || (nextTok.tt == tokenTypeEnum::R_PAREN) {
                        break; // Stop loop when semicolon or parentheses is found
                    }
                
                    k += 1;
                }
                // curStmt.push(nextTok);
                // println!("Found the semicolon");

                // println!("CurStmt[1]: {}", curStmt[1].tokenString);
                match curStmt[1].tt {
                    tokenTypeEnum::SET_EQUALS => {
                        let varName = &curStmt[0].tokenString;
                        // println!("command length: {}", &curStmt.len().to_string());
                        //Simple assign
                        if (curStmt.len() == 4) {
                            // println!("Simple set equals found");
                            let varName = curStmt[0].tokenString.clone();
                            let valueRes = Expr::new(curStmt[2].tt.clone(), Some(curStmt[2].tokenString.clone()));
                            let mut valueExpr:Expr; 
                            match valueRes {
                                Ok(expr) => {
                                    valueExpr = expr;
                                }
                                Err(err) => {
                                    println!("Error creating expression");
                                    let errMsg = format!("Error on line {}: {}", curStmt[0].lineNum, err);
                                    self.reports.reportError(errMsg);
                                    return Err("Error with expression".to_string());
                                }
                            }
                            let assignStmt = Stmt::Assign(varName, valueExpr);
                            tokenList.drain(0..k+1);
                            return Ok(Some(assignStmt));
                            
                        }  else if (curStmt.len() > 4) {
                            // println!("complex set equals");
    
                            let mut subList = tokenList.clone();
                            subList.drain(0..2); 
                            // println!("First token: {}", subList[0].tokenString);
                            let mut parsedExpr: Expr;
                            let scanned = self.parse(&mut subList);                            
                                let mut headerStmt:Expr;
                                // let mut headerReporting = Reporting::new();
                                match scanned {
                                    Ok((Some(stmt))) => {
                                        let parsed = stmt.extractExpr();
                                        match parsed {
                                            Ok(expr) => {
                                                parsedExpr = expr
                                            },
                                            Err(msg) => {
                                                println!("Error parsing expression from statment");
                                                let errMsg = format!("Error parsing body: {:?}", self.reports);
                                                parsedExpr = Expr::IntLiteral(0);
                                            }
                                        }
                                                 
                                        
                                    },
                                    Ok((None)) => {
                                        println!("Parsed complex expression but no statement returned.");
                                        parsedExpr = Expr::IntLiteral(0);
                                    },
                                    Err(reporting) => {
                                        println!("Error parsing expression: {:?}", reporting);
                                        let errMsg = format!("Error parsing body: {:?}", self.reports);
    
                                        return Err(errMsg);
                                    },
                                }
                            // println!("Expression parsed: {}", parsedExpr);
                            let retStmt = Stmt::Assign(varName.to_string(), parsedExpr);
                            
                            // parsedStmt.display(0);
                            tokenList.drain(0..k+1);
                            return Ok(Some(retStmt));
                        
                        } else {
                            // println!("{}", curStmt[1].tt);
                            println!("Fuck you");
                            self.reports.reportError(format!(
                                "In line: {}, Satement is too short'", curStmt[3].lineNum));
                            return Err("Error with identifier".to_string());
                        } 
                    }
                    tokenTypeEnum::L_BRACKET => {
                        println!("Left bracket found");
                        if(curStmt[3].tt != tokenTypeEnum::R_BRACKET){
                            println!("No right bracket, has this: {}", curStmt[3].tokenString);
                            self.reports.reportError(format!(
                                "In line: {}, Array variable incorrect.", 
                                curStmt[3].lineNum, 
                            ));
                            return Err("Error with variable declaration".to_string());
                        }

                        
                        // println!("Simple set equals found");
                        let varName = curStmt[0].tokenString.clone();
                        // println!("Value: {}", curStmt[5].tokenString);
                        let valueRes = Expr::new(curStmt[5].tt.clone(), Some(curStmt[5].tokenString.clone()));
                        let mut valueExpr:Expr; 
                        match valueRes {
                            Ok(expr) => {
                                valueExpr = expr;
                            }
                            Err(err) => {
                                println!("Error creating array expression");
                                let errMsg = format!("Error with expression on line {}: {}", curStmt[0].lineNum, err);
                                self.reports.reportError(errMsg);
                                return Err("Error with expression".to_string());
                            }
                        }
                        
                        let indexRes = Expr::new(curStmt[2].tt.clone(), Some(curStmt[2].tokenString.clone()));
                        let mut indexExpr:Expr; 
                        match indexRes {
                            Ok(expr) => {
                                indexExpr = expr;
                            }
                            Err(err) => {
                                // println!("Error creating expression");
                                let errMsg = format!("Error with parsing array index on line {}: {}", curStmt[0].lineNum, err);
                                self.reports.reportError(errMsg);
                                return Err("Error with expression".to_string());
                            }
                        }

                        // println!("Variable name: {}", varName.to_string());
                        // println!("Index: {}", indexExpr);
                        // println!("Value: {}", valueExpr);

                        //Converts the guys to boxes
                        let boxIndex: Box<Expr> = Box::new(indexExpr);
                        let boxValue: Box<Expr> = Box::new(valueExpr);

                        let arraySet = Expr::ArrayAssign(boxIndex, boxValue);
                        
                        
                        let assignStmt = Stmt::Assign(varName, arraySet);

                        // println!("Assignment:");
                        // assignStmt.display(0);

                        // println!("Next token: {}", tokenList[k+1].tokenString);
                        tokenList.drain(0..k+1);

                        return Ok(Some(assignStmt));
                    }
                    _ => {
                        // println!("Found an expression of type: {}", curStmt[1].tokenString);
                        // println!("Expressions length: {}", curStmt.len());
                        if(curStmt.len() == 4) {
                            // println!("Simple expression");
                            // println!("First token in simple expression: {}", curStmt[0].tokenString);
                            let operand1 = Expr::new(curStmt[0].tt.clone(), Some(curStmt[0].tokenString.clone()));
                            let mut op1Expr: Expr;
                            match operand1 {
                                Ok(expr) => {
                                    op1Expr = expr;
                                }
                                Err(err) => {
                                    println!("Error parsing operand 1");
                                    let errMsg = format!("Error with operand 1 on line {}: {}", curStmt[0].lineNum, err);
                                    self.reports.reportError(errMsg);
                                    return Err("Error with operand 1".to_string());
                                }
                            }

                            
                            let operand2 = Expr::new(curStmt[2].tt.clone(), Some(curStmt[2].tokenString.clone()));
                            let mut op2Expr: Expr;
                            match operand2 {
                                Ok(expr) => {
                                    op2Expr = expr;
                                }
                                Err(err) => {
                                    println!("Error parsing operand 2");
                                    let errMsg = format!("Error with operand 2 on line {}: {}", curStmt[0].lineNum, err);
                                    self.reports.reportError(errMsg);
                                    return Err("Error with operand 2".to_string());
                                }
                            }

                        
                            let operator = BinOp::new(curStmt[1].tt.clone());
                            let mut opBin:BinOp; 
                            match operator {
                                Ok(expr) => {
                                    opBin = expr;
                                }
                                Err(err) => {
                                    println!("Error creating expression");
                                    let errMsg = format!("Error with operator on line {}: {}", curStmt[0].lineNum, err);
                                    self.reports.reportError(errMsg);
                                    return Err("Error with operator".to_string());
                                }
                            }

                            let finalExpr = Expr::BinOp(Box::new(op1Expr), opBin, Box::new(op2Expr));

                            let retStmt = Stmt::Expr(finalExpr);
                            tokenList.drain(0..k+1);
                            return Ok(Some(retStmt));

                        } else if (curStmt.len() > 4) {
                            // println!("Complex expressions");
                            // println!("First complex expression token: {}", curStmt[0].tokenString);

                            //Parses the first operand
                            let operand1 = Expr::new(curStmt[0].tt.clone(), Some(curStmt[0].tokenString.clone()));
                            let mut op1Expr: Expr;
                            match operand1 {
                                Ok(expr) => {
                                    op1Expr = expr;
                                }
                                Err(err) => {
                                    println!("Error parsing operand 1");
                                    let errMsg = format!("Error with operand 1 on line {}: {}", curStmt[0].lineNum, err);
                                    self.reports.reportError(errMsg);
                                    return Err("Error with operand 1".to_string());
                                }
                            }

                            // println!("Operand 1: {}", op1Expr);
                            
                            let mut subList = tokenList.clone();
                            subList.drain(0..2);
                            // println!("First new token expression token: {}", subList[0].tokenString);

                            let mut parsedExpr: Expr;
                            let scanned = self.parse(&mut subList);                            
                                let mut headerStmt:Expr;
                                // let mut headerReporting = Reporting::new();
                                match scanned {
                                    Ok((Some(stmt))) => {
                                        let parsed = stmt.extractExpr();
                                        match parsed {
                                            Ok(expr) => {
                                                parsedExpr = expr
                                            },
                                            Err(msg) => {
                                                println!("Error parsing expression from statment");
                                                let errMsg = format!("Error parsing body: {:?}", self.reports);
                                                parsedExpr = Expr::IntLiteral(0);
                                            }
                                        }
                                                 
                                        
                                    },
                                    Ok((None)) => {
                                        println!("Parsed complex expression but no statement returned.");
                                        parsedExpr = Expr::IntLiteral(0);
                                    },
                                    Err(reporting) => {
                                        println!("Error parsing expression: {:?}", reporting);
                                        let errMsg = format!("Error parsing body: {:?}", self.reports);
    
                                        return Err(errMsg);
                                    },
                                }
                            // println!("Expression parsed: {}", parsedExpr);
                            let op2Expr = parsedExpr;
                            // println!("Operand 2: {}", op2Expr);


                            let operator = BinOp::new(curStmt[1].tt.clone());
                            let mut opBin:BinOp; 
                            match operator {
                                Ok(expr) => {
                                    opBin = expr;
                                }
                                Err(err) => {
                                    println!("Error creating expression");
                                    let errMsg = format!("Error with operator on line {}: {}", curStmt[0].lineNum, err);
                                    self.reports.reportError(errMsg);
                                    return Err("Error with operator".to_string());
                                }
                            }

                            // println!("Operator: {}", opBin);

                            
                            let finalExpr = Expr::BinOp(Box::new(op1Expr), opBin, Box::new(op2Expr));

                            // println!("Final complex expression: {}", finalExpr);

                            let retStmt = Stmt::Expr(finalExpr);
                            tokenList.drain(0..k+1);
                            return Ok(Some(retStmt));
                        } else {
                            println!("Fucked up expressions");
                             // println!("{}", curStmt[1].tt);
                             self.reports.reportError(format!(
                                 "In line: {}, expression is too short'", curStmt[3].lineNum));
                             return Err("Error with expression".to_string());
                        }
                    }
                }
            }
            
            