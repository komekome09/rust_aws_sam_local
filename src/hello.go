package main

import (
    "fmt"
    "bytes"
    "context"
    "strings"
    "net/http"
    "github.com/aws/aws-lambda-go/lambda"
    "github.com/PuerkitoBio/goquery"
)

func SearchRestNumbers(url string) (string, error) {
    var text string

    doc, err := goquery.NewDocument(url)
    if err != nil {
        return "Cannot get https://www.amazon.co.jp/dp/B079211FWH/", err
    }

    doc.Find("span#productTitle").Each(func(_ int, s *goquery.Selection) {
        text = s.Text()
        text = strings.TrimSpace(text)
    })
    title := text

    doc.Find("span#priceblock_ourprice").Each(func(_ int, s *goquery.Selection) {
        text = s.Text()
        text = strings.TrimSpace(text)
    })
    price := text

    doc.Find("#availability > span:nth-child(1)").Each(func(_ int, s *goquery.Selection) {
        text = s.Text()
        text = strings.TrimSpace(text)
    })
    rest := text
    response := title + "[" + price + "] " + rest

    jsonStr := `{"text":"` + response + `"}`

    req, err := http.NewRequest(
        "POST",
        "https://hooks.slack.com/services/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",
        bytes.NewBuffer([]byte(jsonStr)),
    )
    if err != nil{
        return "Cannot create post to slack", err
    }

    req.Header.Set("Content-Type", "application/json")

    client := &http.Client{}
    resp, err := client.Do(req)
    if err != nil{
        return "Cannnot post to slack", err
    }

    defer resp.Body.Close()

    return response, nil
}

func HandleRequest(ctx context.Context) (string, error) {
    response, err := SearchRestNumbers("https://www.amazon.co.jp/gp/product/B079211FWH/ref=crt_ewc_title_dp_1?ie=UTF8&psc=1&smid=AN1VRQENFRJN5")
    return fmt.Sprintf("%s", response), err
}

func main(){
    lambda.Start(HandleRequest)
}
