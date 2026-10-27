<?php 
	/*	
	*	Goodlayers Core Prebuilt Block
	*/
	
	add_filter('gdlr_core_page_builder_block_template_list', 'gdlr_core_page_builder_block_template_list');
	add_filter('gdlr_core_page_builder_get_block_template', 'gdlr_core_page_builder_get_block_template', 10, 2);

	if( !function_exists('gdlr_core_page_builder_block_template_list') ){
		function gdlr_core_page_builder_block_template_list( $templates ){
			$templates['about-us'] = array(
				'title' => esc_html__('About Us', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/about-us.jpg',
			);	
			$templates['about-us-2'] = array(
				'title' => esc_html__('About Us 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/about-us-2.jpg',
			);	
			$templates['about-us-3'] = array(
				'title' => esc_html__('About Us 3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/about-us-3.jpg',
			);				
			$templates['about-us-4'] = array(
				'title' => esc_html__('About Us 4', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/about-us-4.jpg',
			);	
			$templates['about-us-5'] = array(
				'title' => esc_html__('About Us 5', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/about-us-5.jpg',
			);	
			$templates['about-with-number'] = array(
				'title' => esc_html__('About With Number', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/about-with-number.jpg',
			);
			$templates['basic-column-services'] = array(
				'title' => esc_html__('Basic Column Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/basic-column-services.jpg',
			);			
			$templates['basic-column-services-2'] = array(
				'title' => esc_html__('Basic Column Services 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/basic-column-services-2.jpg',
			);
			$templates['basic-counters'] = array(
				'title' => esc_html__('Basic Counters', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/basic-counters.jpg',
			);
			$templates['big-column-services'] = array(
				'title' => esc_html__('Big Column Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/big-column-services.jpg',
			);
			$templates['column-services-images'] = array(
				'title' => esc_html__('Column Services Images', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/column-services-images.jpg',
			);
			$templates['column-services-1'] = array(
				'title' => esc_html__('Column Services 1', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/column-service-1.jpg',
			);
			$templates['column-services-2'] = array(
				'title' => esc_html__('Column Services 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/column-services-2.jpg',
			);
			$templates['column-services-3'] = array(
				'title' => esc_html__('Column Services 3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/column-services-3.jpg',
			);
			$templates['column-services-x3'] = array(
				'title' => esc_html__('Column Services X3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/column-services-x3.jpg',
			);
			$templates['column-service-lite'] = array(
				'title' => esc_html__('Column Service Lite', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/column-service-lite.jpg',
			);
			$templates['column-service-number'] = array(
				'title' => esc_html__('Column Service Lite', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/column-service-number.jpg',
			);
			$templates['contact-detail'] = array(
				'title' => esc_html__('Contact Detail', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/contact-detail.jpg',
			);
			$templates['contact-detail-2'] = array(
				'title' => esc_html__('Contact Detail 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/contact-detail-2.jpg',
			);
			$templates['contact-form'] = array(
				'title' => esc_html__('Contact Form', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/contact-form.jpg',
			);
			$templates['content-list'] = array(
				'title' => esc_html__('Content List', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/content-list.jpg',
			);
			$templates['content-list-left'] = array(
				'title' => esc_html__('Content List Left', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/content-list-left.jpg',
			);
			$templates['content-preset-1'] = array(
				'title' => esc_html__('Content Preset 1', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/content-preset-1.jpg',
			);
			$templates['content-preset-2'] = array(
				'title' => esc_html__('Content Preset 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/content-preset-2.jpg',
			);
			$templates['content-preset-3'] = array(
				'title' => esc_html__('Content Preset 3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/content-preset-3.jpg',
			);
			$templates['content-preset-4'] = array(
				'title' => esc_html__('Content Preset 4', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/content-preset-4.jpg',
			);
			$templates['content-preset-5'] = array(
				'title' => esc_html__('Content Preset 5', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/content-preset-5.jpg',
			);
			$templates['counter'] = array(
				'title' => esc_html__('Counter', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/counter.jpg',
			);
			$templates['counter2'] = array(
				'title' => esc_html__('Counter 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/counter2.jpg',
			);
			$templates['counter-shadow-box'] = array(
				'title' => esc_html__('Counter Shadow Box', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/counter-shadow-box.jpg',
			);
			$templates['counter-box-x4'] = array(
				'title' => esc_html__('Counter Box X4', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/counter-box-x4.jpg',
			);
			$templates['counter-with-cta'] = array(
				'title' => esc_html__('Counter With CTA', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/counter-with-cta.jpg',
			);
			$templates['custom-contact-1'] = array(
				'title' => esc_html__('Custom Contact 1', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/custom-contact-1.jpg',
			);
			$templates['custom-counters'] = array(
				'title' => esc_html__('Custom Counters', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/custom-counters.jpg',
			);
			$templates['custom-footer'] = array(
				'title' => esc_html__('Custom Footer', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/custom-footer.jpg',
			);
			$templates['custom-map'] = array(
				'title' => esc_html__('Custom Map', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/custom-map.jpg',
			);
			$templates['custom-services-1'] = array(
				'title' => esc_html__('Custom Services 1', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/custom-services-1.jpg',
			);
			$templates['custom-services-2'] = array(
				'title' => esc_html__('Custom Services 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/custom-services-2.jpg',
			);
			$templates['custom-services-3'] = array(
				'title' => esc_html__('Custom Services 3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/custom-services-3.jpg',
			);
			$templates['customer-says'] = array(
				'title' => esc_html__('Customer Says', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/customer-says.jpg',
			);
			$templates['faq'] = array(
				'title' => esc_html__('FAQ', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/faq.jpg',
			);
			$templates['faq-tab'] = array(
				'title' => esc_html__('FAQ Tab', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/faq-tab.jpg',
			);
			$templates['featured-boxes'] = array(
				'title' => esc_html__('Featured Boxes', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/featured-boxes.jpg',
			);
			$templates['featured-content'] = array(
				'title' => esc_html__('Featured Content', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/featured-content.jpg',
			);
			$templates['featured-content-skew'] = array(
				'title' => esc_html__('Featured Content Skew', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/featured-content-skew.jpg',
			);
			$templates['featured-video'] = array(
				'title' => esc_html__('Featured Video', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/featured-video.jpg',
			);
			$templates['flip-boxes'] = array(
				'title' => esc_html__('Flip Boxes', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/flip-boxes.jpg',
			);
			$templates['flip-boxes-x3'] = array(
				'title' => esc_html__('Flip Boxes X3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/flip-boxes-x3.jpg',
			);
			$templates['flip-boxes-accordion'] = array(
				'title' => esc_html__('Flip Boxes Accordion', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/flip-boxes-accordion.jpg',
			);
			$templates['half-about'] = array(
				'title' => esc_html__('Half About', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-about.jpg',
			);
			$templates['half-about2'] = array(
				'title' => esc_html__('Half About 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-about2.jpg',
			);
			$templates['half-about3'] = array(
				'title' => esc_html__('Half About 3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-about3.jpg',
			);
			$templates['half-about4'] = array(
				'title' => esc_html__('Half About 4', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-about4.jpg',
			);
			$templates['half-accordion'] = array(
				'title' => esc_html__('Half Accordion', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-accordion.jpg',
			);
			$templates['half-box-list'] = array(
				'title' => esc_html__('Half Box List', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-box-list.jpg',
			);
			$templates['half-box-services'] = array(
				'title' => esc_html__('Half Box Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-box-services.jpg',
			);
			$templates['half-box-services2'] = array(
				'title' => esc_html__('Half Box Services 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-box-services2.jpg',
			);
			$templates['half-box-x3'] = array(
				'title' => esc_html__('Half Box X3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-box-x3.jpg',
			);
			$templates['half-box-x3-2'] = array(
				'title' => esc_html__('Half Box X3-2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-box-x3-2.jpg',
			);
			$templates['half-box-x3-3'] = array(
				'title' => esc_html__('Half Box X3-3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-boxesx3-3.jpg',
			);
			$templates['half-boxed-video'] = array(
				'title' => esc_html__('Half Box Video', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-boxed-video.jpg',
			);
			$templates['half-column-servicex4'] = array(
				'title' => esc_html__('Half Column Service X4', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-col-servicex4.jpg',
			);
			$templates['half-contact'] = array(
				'title' => esc_html__('Half Contact', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-contact.jpg',
			);
			$templates['half-contact-2'] = array(
				'title' => esc_html__('Half Contact 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-contact-2.jpg',
			);
			$templates['image-boxes-x3'] = array(
				'title' => esc_html__('Image Boxes X3', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/image-boxes-x3.jpg',
			);
			$templates['it-solutions'] = array(
				'title' => esc_html__('IT Solutions', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/it-solutions.jpg',
			);
			$templates['large-quote'] = array(
				'title' => esc_html__('Large Quote', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/large-quote.jpg',
			);
			$templates['left-mobile-services'] = array(
				'title' => esc_html__('Left Mobile Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/left-mobile-services.jpg',
			);
			$templates['modern-services'] = array(
				'title' => esc_html__('Modern Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/modern-services.jpg',
			);
			$templates['price-table'] = array(
				'title' => esc_html__('Price Table', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/price-table.jpg',
			);
			$templates['quote-half-box'] = array(
				'title' => esc_html__('Quote Half Box', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/quote-half-box.jpg',
			);
			$templates['quote-with-video'] = array(
				'title' => esc_html__('Quote With Video', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/quote-with-video.jpg',
			);
			$templates['right-display-services'] = array(
				'title' => esc_html__('Right Display Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/right-display-services.jpg',
			);
			$templates['service-boxes-x4'] = array(
				'title' => esc_html__('Service Boxes X4', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/service-boxes-x4.jpg',
			);
			$templates['testimonial-shadow-box'] = array(
				'title' => esc_html__('Testimonial Shadow Box', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/testimonial-shadow-box.jpg',
			);
			$templates['text-with-skills'] = array(
				'title' => esc_html__('Text With Skills', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/text-with-skills.jpg',
			);
			$templates['three-boxes-info'] = array(
				'title' => esc_html__('Three Boxes Info', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/three-boxes-info.jpg',
			);
			$templates['token-allocation'] = array(
				'title' => esc_html__('Token Allocation', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/token-allocation.jpg',
			);
			$templates['seo-about'] = array(
				'title' => esc_html__('SEO About', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/seo-about.jpg',
			);
			$templates['seo-services'] = array(
				'title' => esc_html__('SEO Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/seo-services.jpg',
			);
			$templates['service-boxes'] = array(
				'title' => esc_html__('Service Boxes', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/service-boxes.jpg',
			);
			$templates['service-circle-icons'] = array(
				'title' => esc_html__('Service Circle Icons', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/service-circle-icons.jpg',
			);
			$templates['skill-circle'] = array(
				'title' => esc_html__('Skill Circle', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/skill-circle.jpg',
			);
			$templates['simple-timeline'] = array(
				'title' => esc_html__('Simple Timeline', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/simple-timeline.jpg',
			);
			$templates['sponsors'] = array(
				'title' => esc_html__('Sponsors', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/sponsors.jpg',
			);
			$templates['stunning-service'] = array(
				'title' => esc_html__('Stunning Service', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/stunning-service.jpg',
			);
			$templates['icon-services'] = array(
				'title' => esc_html__('Icon Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/icon-services.jpg',
			);
			$templates['icon-list'] = array(
				'title' => esc_html__('Icon List', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/icon-list.jpg',
			);
			$templates['icon-list2'] = array(
				'title' => esc_html__('Icon List 2', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/icon-list2.jpg',
			);
			$templates['ico-service'] = array(
				'title' => esc_html__('Ico Service', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/ico-service.jpg',
			);
			$templates['half-services'] = array(
				'title' => esc_html__('Half Services', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/half-services.jpg',
			);
			$templates['case-studies'] = array(
				'title' => esc_html__('Case Studies', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/case-studies.jpg',
			);
			$templates['video-about'] = array(
				'title' => esc_html__('Video About', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/video-about.jpg',
			);
			$templates['why-us-image'] = array(
				'title' => esc_html__('Why Us Image', 'goodlayers-core'),
				'type' => 'wrapper',
				'thumbnail' => GDLR_CORE_URL . '/template/images/block/why-us-image.jpg',
			);
			return $templates;

		} // gdlr_core_page_builder_block_template_list
	} // function_exists

	if( !function_exists('gdlr_core_page_builder_get_block_template') ){
		function gdlr_core_page_builder_get_block_template( $value, $type ){

			if( !empty($value) ) return $value;

			$file = GDLR_CORE_LOCAL . '/template/data/' . $type . '.json';
			if( file_exists($file) ){
				return file_get_contents($file);
			}

		} // gdlr_core_page_builder_get_block_template
	} // function_exists