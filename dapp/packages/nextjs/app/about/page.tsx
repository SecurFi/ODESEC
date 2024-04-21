"use client";
import type { NextPage } from "next";
import Image from "next/image";


const About: NextPage = () => {


  return (

    <div className="flex flex-col items-center flex-grow mt-10">
      <h1>How it works?</h1>
      <div className="mt-5 min-h-[900px]">
        <Image alt="protocol diagram" height={1000} width={900} src="/diagram.svg" />
      </div>

    </div>


  );
};

export default About;
