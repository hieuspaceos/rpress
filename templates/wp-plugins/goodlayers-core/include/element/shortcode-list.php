<?php
	/*	
	*	Goodlayers Item
	*/

	if( is_admin() ){ add_filter('gdlr_core_shortcode_list', 'gdlr_core_register_shortcode_list'); }
	if( !function_exists('gdlr_core_register_shortcode_list') ){
		function gdlr_core_register_shortcode_list( $shortcode_list ){
			$shortcode_list = array_merge($shortcode_list, array(
				array(
					'title' => 'Blog',
					'value' => '[gdlr_core_blog num-fetch="3" blog-style="blog-widget" category="" thumbnail-size="thumbnail" ]'
				),
				array(
					'title' => 'Button',
					'value' => '[gdlr_core_button button-text="Learn More" button-link="#" button-link-target="_blank" margin-right="20px" ]'
				),
				array(
					'title' => 'Code',
					'value' => '[gdlr_core_code style="light" ]<br>' . 
						'code content<br>' . 
						'[/gdlr_core_code]'
				),
				array(
					'title' => 'Column',
					'value' => '[gdlr_core_row]<br>' . 
						'[gdlr_core_column size="1/3"]column content 1[/gdlr_core_column]<br>' .
						'[gdlr_core_column size="1/3"]column content 2[/gdlr_core_column]<br>' .
						'[gdlr_core_column size="1/3"]column content 3[/gdlr_core_column]<br>' .
						'[/gdlr_core_row]'
				),
				array(
					'title' => 'Dropcap',
					'value' => '[gdlr_core_dropcap type="circle" color="#ffffff" background="#212121"]S[/gdlr_core_dropcap]'
				),
				array(
					'title' => 'Dropdown Tab',
					'value' => '[gdlr_core_dropdown_tab]<br>' . 
						'[gdlr_core_tab title="TITLE 1" ]CONTENT 1[/gdlr_core_tab]<br>' .
						'[gdlr_core_tab title="TITLE 2" ]CONTENT 2[/gdlr_core_tab]<br>' .
						'[gdlr_core_tab title="TITLE 3" ]CONTENT 3[/gdlr_core_tab]<br>' .
						'[/gdlr_core_dropdown_tab]'
				),
				array(
					'title' => 'Gallery',
					'value' => '[gallery ids="875,874,873,876,877" source="gdlr-core" style="slider" slider-navigation="bullet" thumbnail-size="thumbnail" ]'
				),
				array(
					'title' => 'Icon',
					'value' => '[gdlr_core_icon icon="" size="" color="" margin-left="" margin-right="" ]'
				),
				array(
					'title' => 'Port Info',
					'value' => '[gdlr_core_port_info]<br>' .
						'[gdlr_core_tab title="key" ]value[/gdlr_core_tab]<br>' .
						'[gdlr_core_tab title="key" ]value[/gdlr_core_tab]<br>' .
						'[gdlr_core_tab title="key" ]value[/gdlr_core_tab]<br>' .
						'[/gdlr_core_port_info]'
				),
				array(
					'title' => 'Social Network',
					'value' => '[gdlr_core_social_network facebook="#url" email="#url" twitter="#url" ]'
				),
				array(
					'title' => 'Space',
					'value' => '[gdlr_core_space height="30px"]'
				),
				array(
					'title' => 'Tab',
					'value' => '[gdlr_core_tabs]<br>' .
						'[gdlr_core_tab title="title 1"]Tab 1[/gdlr_core_tab]<br>' .
						'[gdlr_core_tab title="title 2"]Tab 2[/gdlr_core_tab]<br>' .
						'[gdlr_core_tab title="title 3"]Tab 3[/gdlr_core_tab]<br>' .
						'[/gdlr_core_tabs]'
				),
				array(
					'title' => 'Table',
					'value' => 
						'<table>' .
						'<tbody>' .
						'<tr><th>Pharetra</th><th>Malesuada</th><th>Cursus</th><th>Euismod</th></tr>' .
						'<tr><td>Ipsum</td><td>Portalion</td><td>Elitesimo</td><td>Aenean</td></tr>' .
						'<tr><td>Ipsum</td><td>Portalion</td><td>Elitesimo</td><td>Aenean</td></tr>' .
						'<tr><td>Ipsum</td><td>Portalion</td><td>Elitesimo</td><td>Aenean</td></tr>' .
						'<tr><td>Ipsum</td><td>Portalion</td><td>Elitesimo</td><td>Aenean</td></tr>' .
						'<tr><td>Ipsum</td><td>Portalion</td><td>Elitesimo</td><td>Aenean</td></tr>' .
						'</tbody>' .
						'</table>'
				),
				array(
					'title' => 'Title',
					'value' => '[gdlr_core_title title="" caption="" ]'
				),
				array(
					'title' => 'Widget Box',
					'value' => '[gdlr_widget_box title="Get a Question?" title-color="#ffffff" background="#252525" color="#ffffff" ]<br>' . 
						'Do not hesitage to give us a call. We are an expert team and we are happy to talk to you.<br>' . 
						'[/gdlr_widget_box]'
				),
				array(
					'title' => 'Widget List',
					'value' => '[gdlr_widget_list title="Why Book With Us?" title-color="" background-color="" color="" border-color="" ]<br>' . 
						'<ul>' . 
						'<li>No-hassle best price guarantee</li>' . 
						'<li>Customer care available 24/7</li>' . 
						'<li>Hand-picked Tours & Activities</li>' . 
						'<li>Free Travel Insureance</li>' . 
						'</ul>' . 
						'[/gdlr_widget_list]'
				),
			));

			return $shortcode_list;
		}
	}